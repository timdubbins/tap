use std::ops::Range;
use std::path::PathBuf;

use rand::{thread_rng, Rng};

pub fn has_child(path: &PathBuf) -> bool {
    let iter = match path.read_dir() {
        Ok(r) => r,
        Err(_) => return false,
    };
    for entry in iter {
        if let Ok(entry) = entry {
            if entry.path().is_dir() {
                return true;
            }
        }
    }
    false
}

// Returns true if the path has at least two children.
pub fn has_child_dirs(path: &PathBuf) -> bool {
    let mut has_child_dir = false;

    let iter = match path.read_dir() {
        Ok(r) => r,
        Err(_) => return false,
    };

    for entry in iter {
        if let Ok(entry) = entry {
            if entry.path().is_dir() {
                if has_child_dir {
                    // The second child is found.
                    return true;
                } else {
                    // The first child is found.
                    has_child_dir = true;
                }
            }
        }
    }
    false
}

// Maps the array to a single value, i.e. `[0, 1, 2]` -> `12`.
pub fn concatenate(arr: &Vec<usize>) -> usize {
    arr.iter().fold(0, |acc, x| acc * 10 + x)
}

// Generates a random unsigned int in the given range.
pub fn random(range: Range<usize>) -> usize {
    thread_rng().gen_range(range)
}
