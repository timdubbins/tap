use std::{cmp::Ordering, path::PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::utils::has_child;

#[derive(Clone, Eq, PartialEq, Ord)]
pub struct FuzzyItem {
    // The path of the directory entry.
    pub path: PathBuf,
    // The depth of the directory, relative to initial `path`.
    pub depth: usize,
    // The file name of the directory entry.
    pub display: String,
    // The first character of `display`, uppercased.
    pub key: char,
    // Whether or not `path` contains subdirectories.
    pub has_child: bool,
    // The indices of `display` that are fuzzy matched.
    pub indices: Vec<usize>,
    // The weight of the fuzzy match. Better matches have higher weight.
    pub weight: i64,
}

impl FuzzyItem {
    pub fn new(dent: DirEntry) -> Self {
        let path = dent.path().into();

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

        FuzzyItem {
            has_child: has_child(&path),
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
        }
    }
}

// Use alphabetical ordering.
impl PartialOrd for FuzzyItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.key.cmp(&other.key))
    }
}

// Builds the list of fuzzy items from the non-hidden subdirectories of `path`.
pub fn get_items(path: &PathBuf) -> Vec<FuzzyItem> {
    let items = WalkDir::new(path)
        .min_depth(1)
        .into_iter()
        .filter_entry(is_non_hidden_dir)
        .filter_map(|res| res.ok())
        .map(|dent| FuzzyItem::new(dent))
        .collect::<Vec<FuzzyItem>>();

    // Exclude single items so we can load them without fuzzy matching.
    match items.len() {
        1 => vec![],
        _ => items,
    }
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

// Gets all the non-leaf items that start with the letter `key`.
pub fn key_items(key: char, items: Vec<FuzzyItem>) -> Vec<FuzzyItem> {
    items
        .into_iter()
        .filter(|e| e.has_child && e.key == key)
        .collect()
}

// Gets all the items that are `depth` level directories, sorted alphabetically.
pub fn depth_items(depth: usize, items: Vec<FuzzyItem>) -> Vec<FuzzyItem> {
    let mut items = items
        .into_iter()
        .filter(|e| e.depth == depth)
        .collect::<Vec<FuzzyItem>>();
    items.sort();
    items
}

// Gets all the non-leaf items, sorted alphabetically.
pub fn non_leaf_items(items: Vec<FuzzyItem>) -> Vec<FuzzyItem> {
    let mut items = items
        .into_iter()
        .filter(|e| e.has_child)
        .collect::<Vec<FuzzyItem>>();
    items.sort();
    items
}

// Gets all the leaf items, sorted alphabetically.
pub fn leaf_items(items: Vec<FuzzyItem>) -> Vec<FuzzyItem> {
    let mut items = items
        .into_iter()
        .filter(|e| !e.has_child)
        .collect::<Vec<FuzzyItem>>();
    items.sort();
    items
}

// Gets all the leaf paths.
pub fn leaf_paths(items: &Vec<FuzzyItem>) -> Vec<PathBuf> {
    items
        .into_iter()
        .filter(|e| !e.has_child)
        .map(|e| e.path.to_owned())
        .collect::<Vec<PathBuf>>()
}
