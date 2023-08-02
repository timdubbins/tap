use std::{ops::Range, os::unix::prelude::OsStrExt, path::PathBuf};

use anyhow::bail;
use rand::{thread_rng, Rng};

// Returns true if the path has at least two children.
pub fn has_child_dirs(path: &PathBuf) -> bool {
    let mut has_child_dir = false;

    if path.is_dir() {
        for entry in path.read_dir().expect("directory should not be empty") {
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
    } else {
        return false;
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

// Returns the PathBuf on success.
pub fn remove_trailing_slash(p: PathBuf) -> Result<PathBuf, anyhow::Error> {
    if has_trailing_slash(&p) {
        match p.clone().into_os_string().into_string() {
            Ok(s) => {
                let mut s = s;
                s.pop();
                Ok(PathBuf::from(s))
            }
            Err(_) => bail!("Couldn't remove trailing slash from '{}'", p.display()),
        }
    } else {
        Ok(p)
    }
}

// Whether or not the last character is a slash.
fn has_trailing_slash(p: &PathBuf) -> bool {
    p.as_os_str().as_bytes().last() == Some(&b'/')
}
