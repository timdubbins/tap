use std::{cmp::Ordering, path::PathBuf};

use {
    anyhow::{anyhow, bail},
    bincode::{Decode, Encode},
    serde::{Deserialize, Serialize},
    walkdir::DirEntry,
};

use crate::{player::AudioFile, TapError};

// A struct representing a directory containing audio files, with metadata used for
// fuzzy matching and filtering.
#[derive(Clone, Debug, Eq, PartialEq, Ord, Encode, Decode, Serialize, Deserialize, Default)]
pub struct FuzzyDir {
    // The file name of the directory entry.
    pub name: String,
    // Indices of characters in `name` that are matched during fuzzy searching.
    pub match_indices: Vec<usize>,
    // The weight of the fuzzy match; better matches have higher weights.
    pub match_weight: i64,
    // The full path of the directory entry on the file system.
    pub path: PathBuf,
    // The depth of the directory relative to the initial search root.
    pub depth: usize,
    // The first character of the `name`, converted to uppercase, used as a key for filtering.
    pub key: char,
    // Whether or not this directory contains an audio file.
    pub contains_audio: bool,
    // Whether or not this directory contains a subdirectory.
    pub contains_subdir: bool,
}

impl TryFrom<PathBuf> for FuzzyDir {
    type Error = TapError;

    fn try_from(path: PathBuf) -> Result<Self, TapError> {
        let mut dir = FuzzyDir::default();
        dir.name = path
            .file_name()
            .and_then(|os_str| os_str.to_str().map(String::from))
            .ok_or(anyhow!("Invalid file name"))?;

        Ok(dir)
    }
}

impl FuzzyDir {
    pub fn new(entry: DirEntry) -> Result<Self, TapError> {
        let (contains_audio, contains_subdir) = Self::check_audio_and_subdirs(&entry)?;

        let name = entry
            .file_name()
            .to_os_string()
            .into_string()
            .unwrap_or_default();

        let key = name.chars().next().unwrap_or_default().to_ascii_uppercase();

        let audio_dir = FuzzyDir {
            name,
            key,
            contains_audio,
            contains_subdir,
            depth: entry.depth(),
            path: entry.into_path(),
            match_indices: vec![],
            // Assign an initial non-zero weight to ensure the directory
            // is included in the results before fuzzy matching.
            match_weight: 1,
        };

        Ok(audio_dir)
    }

    // Checks whether the directory contains at least one audio file or subdirectory.
    fn check_audio_and_subdirs(entry: &DirEntry) -> Result<(bool, bool), TapError> {
        let mut contains_audio = false;
        let mut contains_subdir = false;

        for entry in entry.path().read_dir()? {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    contains_subdir = true;
                } else if !contains_audio {
                    contains_audio = AudioFile::validate_format(&entry.path());
                }
            }

            if contains_audio && contains_subdir {
                break;
            }
        }

        if entry.depth() == 0 {
            if contains_audio {
                contains_subdir = false;
            } else {
                bail!("No audio in root dir")
            }
        }

        if contains_audio || contains_subdir {
            Ok((contains_audio, contains_subdir))
        } else {
            bail!("No audio files or subdirectories found!")
        }
    }

    pub fn is_parent_of(&self, other: &FuzzyDir) -> bool {
        other.path.starts_with(&self.path) && other.path != self.path
    }

    fn is_hidden(entry: &walkdir::DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map_or(false, |s| s.starts_with('.'))
    }

    pub fn is_visible_dir(entry: &walkdir::DirEntry) -> bool {
        entry.file_type().is_dir() && !Self::is_hidden(entry)
    }
}

impl<'a> FromIterator<&'a FuzzyDir> for Vec<FuzzyDir> {
    fn from_iter<I: IntoIterator<Item = &'a FuzzyDir>>(iter: I) -> Self {
        iter.into_iter().cloned().collect()
    }
}

impl PartialOrd for FuzzyDir {
    // Case-insensitive, optimized.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.key.cmp(&other.key) {
            Ordering::Equal => Some(
                self.name
                    .chars()
                    .skip(1)
                    .map(|c| c.to_ascii_lowercase())
                    .cmp(other.name.chars().skip(1).map(|c| c.to_ascii_lowercase())),
            ),
            other_order => Some(other_order),
        }
    }
}
