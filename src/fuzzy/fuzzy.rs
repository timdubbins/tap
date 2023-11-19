use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use anyhow::bail;
use bincode::{Decode, Encode};
use walkdir::{DirEntry, WalkDir};

use crate::player::valid_audio_ext;

#[derive(Clone, Debug, Eq, PartialEq, Ord, Encode, Decode)]
pub struct FuzzyItem {
    // The path of the directory entry.
    pub path: PathBuf,
    // The depth of the directory, relative to initial `path`.
    pub depth: usize,
    // The file name of the directory entry.
    pub display: String,
    // The first character of `display`, uppercased.
    pub key: char,
    // Whether or not the `path` contains audio.
    pub has_audio: bool,
    // The subdirectory count.
    pub child_count: usize,
    // The indices of `display` that are fuzzy matched.
    pub indices: Vec<usize>,
    // The weight of the fuzzy match. Better matches have higher weight.
    pub weight: i64,
}

impl FuzzyItem {
    fn new(res: Result<DirEntry, walkdir::Error>) -> Result<Self, anyhow::Error> {
        let dent = res?;
        let path = dent.path().into();
        let depth = dent.depth();

        // Add the search root as a FuzzyItem iff it contains audio files.
        let (has_audio, sub_dirs) = match depth {
            0 => (has_audio(&path)?, 0),
            _ => validate(&path)?,
        };

        let display = dent
            .file_name()
            .to_os_string()
            .into_string()
            .unwrap_or_default();

        let key = display
            .chars()
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();

        let fuzzy_item = FuzzyItem {
            has_audio,
            child_count: sub_dirs,
            indices: vec![],
            // We assign a default weight so that the weights of
            // items are equal before fuzzy matching. The weight
            // should be non-zero since zero weights are excluded
            // from being displayed. So we choose the value one.
            weight: 1,
            path,
            depth,
            display,
            key,
        };

        Ok(fuzzy_item)
    }
}

impl<'a> FromIterator<&'a FuzzyItem> for Vec<FuzzyItem> {
    fn from_iter<I: IntoIterator<Item = &'a FuzzyItem>>(iter: I) -> Self {
        iter.into_iter().cloned().collect()
    }
}

// Use alphabetical ordering.
impl PartialOrd for FuzzyItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.key.cmp(&other.key))
    }
}

// Creates the list of fuzzy items from the non-hidden subdirectories of `path`.
pub fn create_items(path: &PathBuf) -> Result<Vec<FuzzyItem>, anyhow::Error> {
    let items = WalkDir::new(path)
        .into_iter()
        .filter_entry(is_non_hidden_dir)
        .filter_map(|res| FuzzyItem::new(res).ok())
        .collect::<Vec<FuzzyItem>>();
    Ok(items)
}

// Gets all the non-leaf items that start with the letter `key`.
pub fn key_items(key: Option<char>, items: &Vec<FuzzyItem>) -> Vec<FuzzyItem> {
    if let Some(key) = key {
        items
            .into_iter()
            .filter(|e| e.child_count > 0 && e.key == key)
            .collect()
    } else {
        vec![]
    }
}

// Gets all the items that are `depth` level directories, sorted alphabetically.
pub fn depth_items(depth: usize, items: &Vec<FuzzyItem>) -> Vec<FuzzyItem> {
    let mut items = items
        .into_iter()
        .filter(|e| e.depth == depth)
        .collect::<Vec<FuzzyItem>>();
    items.sort();
    items
}

// Gets all the non-leaf items, sorted alphabetically.
pub fn non_leaf_items(items: &Vec<FuzzyItem>) -> Vec<FuzzyItem> {
    let mut items = items
        .into_iter()
        .filter(|e| e.child_count > 0)
        .collect::<Vec<FuzzyItem>>();
    items.sort();
    items
}

// Returns the path to the directory or file that either contains or is an an audio file,
// if there is only one such directory or file.
pub fn only_audio_path(path: &PathBuf, items: &Vec<FuzzyItem>) -> Option<PathBuf> {
    if items.is_empty() {
        Some(path.to_owned())
    } else {
        let mut path = None;
        for item in items.iter() {
            if item.has_audio {
                if path.is_some() {
                    return None;
                } else {
                    path = Some(item.path.to_owned())
                }
            }
        }
        path
    }
}

// Returns the path to the first directory that contains audio, if any.
pub fn first_audio_path(path: &PathBuf) -> Result<PathBuf, anyhow::Error> {
    let entries = WalkDir::new(path)
        .into_iter()
        .filter_entry(is_non_hidden_dir)
        .filter_map(|entry| entry.ok());

    for entry in entries {
        if let Ok(_) = has_audio(entry.path()) {
            return Ok(path.to_owned());
        }
    }
    bail!("no audio files detected in '{}'", path.display())
}

// Gets all the leaf items, sorted alphabetically.
pub fn audio_items(items: &Vec<FuzzyItem>) -> Vec<FuzzyItem> {
    let mut items = items
        .into_iter()
        .filter(|e| e.has_audio)
        .collect::<Vec<FuzzyItem>>();
    items.sort();
    items
}

// Gets all the leaf paths.
pub fn leaf_paths(items: &Vec<FuzzyItem>) -> Vec<PathBuf> {
    items
        .into_iter()
        .filter(|e| e.has_audio)
        .map(|e| e.path.to_owned())
        .collect::<Vec<PathBuf>>()
}

// Whether the entry is a directory or not. Excludes hidden directories.
fn is_non_hidden_dir(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
        && !entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
}

// Whether or not the path is a directory that contains audio.
fn has_audio<P: AsRef<Path>>(path: P) -> Result<bool, anyhow::Error> {
    for entry in path.as_ref().read_dir()? {
        if let Ok(entry) = entry {
            if valid_audio_ext(&entry.path()) {
                return Ok(true);
            }
        }
    }
    bail!("invalid")
}

// Whether or not a directory is a valid FuzzyItem; that is, does
// the directory contain at least one audio file or child directory.
fn validate(path: &PathBuf) -> Result<(bool, usize), anyhow::Error> {
    let mut has_audio = false;
    let mut dir_count: usize = 0;

    for entry in path.read_dir()? {
        if let Ok(entry) = entry {
            if entry.path().is_dir() {
                dir_count += 1;
            } else if !has_audio {
                has_audio = valid_audio_ext(&entry.path());
            }
        }

        if has_audio && dir_count > 1 {
            break;
        }
    }

    if !has_audio && dir_count == 0 {
        bail!("invalid")
    }

    Ok((has_audio, dir_count))
}
