use std::{cmp::Ordering, path::PathBuf};

use anyhow::bail;
use bincode::{Decode, Encode};
use walkdir::{DirEntry, WalkDir};

use crate::player::is_valid;

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
    pub fn new(res: Result<DirEntry, walkdir::Error>, root: bool) -> Result<Self, anyhow::Error> {
        let dent = res?;
        let path = dent.path().into();

        let (has_audio, sub_dirs) = match root {
            true => (has_audio(&path)?, 0),
            false => validate(&path)?,
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
            depth: dent.depth(),
            indices: vec![],
            // We assign a default weight so that the weights of
            // items are equal before fuzzy matching. The weight
            // should be non-zero since zero weights are excluded
            // from being displayed. So we choose the value one.
            weight: 1,
            path,
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
    let mut items = WalkDir::new(path)
        .min_depth(1)
        .into_iter()
        .filter_entry(is_non_hidden_dir)
        .filter_map(|res| FuzzyItem::new(res, false).ok())
        .collect::<Vec<FuzzyItem>>();

    if let Some(first) = WalkDir::new(path)
        .max_depth(0)
        .contents_first(true)
        .into_iter()
        .filter_map(|res| FuzzyItem::new(res, true).ok())
        .collect::<Vec<_>>()
        .first()
    {
        items.push(first.to_owned())
    }

    Ok(items)
}

// Gets all the non-leaf items that start with the letter `key`.
pub fn key_items(key: char, items: &Vec<FuzzyItem>) -> Vec<FuzzyItem> {
    items
        .into_iter()
        .filter(|e| e.child_count > 0 && e.key == key)
        .collect()
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

fn has_audio(path: &PathBuf) -> Result<bool, anyhow::Error> {
    for entry in path.read_dir()? {
        if let Ok(entry) = entry {
            if is_valid(&entry.path()) {
                return Ok(true);
            }
        }
    }
    bail!("invalid")
}

fn validate(path: &PathBuf) -> Result<(bool, usize), anyhow::Error> {
    let mut has_audio = false;
    let mut dir_count: usize = 0;

    for entry in path.read_dir()? {
        if let Ok(entry) = entry {
            if entry.path().is_dir() {
                dir_count += 1;
            } else if !has_audio {
                has_audio = is_valid(&entry.path());
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
