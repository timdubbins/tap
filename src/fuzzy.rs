use std::{cmp::Ordering, path::PathBuf};

use walkdir::{DirEntry, WalkDir};

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
    // The indices of `display` that are fuzzy matched.
    pub indices: Vec<usize>,
    // The weight of the fuzzy match. Better matches have higher weight.
    pub weight: i64,
}

impl FuzzyItem {
    pub fn new(dent: DirEntry) -> Self {
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
            path: dent.path().into(),
            depth: dent.depth(),
            indices: vec![],
            // We assign a default weight so that the weights of
            // items are equal before fuzzy matching. The weight
            // should be non-zero since zero weights are excluded
            // from being displayed. So we choose the value one.
            weight: 1,
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

// Gets an array of all child directories, relative to `path`,
// excluding hidden directories.
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

// Gets all items with `depth` of 1 and start with `key`.
pub fn filtered_items(key: char, items: Vec<FuzzyItem>) -> Vec<FuzzyItem> {
    items
        .into_iter()
        .filter(|e| e.depth == 1 && e.key == key)
        .collect()
}

// Gets all items with `depth` of 1, sorted by `key`.
pub fn sorted_items(items: Vec<FuzzyItem>) -> Vec<FuzzyItem> {
    let mut items = items
        .into_iter()
        .filter(|e| e.depth == 1)
        .collect::<Vec<FuzzyItem>>();
    items.sort();
    items
}
