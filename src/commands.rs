use std::io::Error;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

use rand::Rng;
use walkdir::{DirEntry, WalkDir};

use crate::app::{App, FuzzyMode};
use crate::theme::{FZF_THEME, SK_THEME};

// Gets an array of all child directories, relative to `path`,
// excluding hidden directories.
pub fn dirs(path: &PathBuf) -> Vec<DirEntry> {
    WalkDir::new(path)
        .min_depth(1)
        .into_iter()
        .filter_entry(is_non_hidden_dir)
        .filter_map(|e| e.ok())
        .collect()
}

// Gets the path of a random subdirectory.
pub fn random_path(app: &App) -> PathBuf {
    let entries = app.dirs.as_ref().unwrap();
    let target = rand::thread_rng().gen_range(0..entries.len() - 1);

    entries[target].to_owned().into_path()
}

// Concatenates the directory file names, delimited by the
// newline character.
pub fn search_string(entries: &Vec<DirEntry>) -> String {
    entries
        .into_iter()
        .map(|e| {
            e.file_name()
                .to_os_string()
                .into_string()
                .unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join("\n")
}

// Gets the fuzzy selected path. If none is selected we return the original path.
pub fn fuzzy_path(app: &App, second_path: Option<PathBuf>, anchor: Option<String>) -> PathBuf {
    // The directories to include in the fuzzy search.
    let (search_string, dirs) = match second_path.clone() {
        Some(second_path) => {
            // The directories to search on, filtered to only those
            // that include `second_path` as a path component.
            let dirs = dirs(&second_path);
            (search_string(&dirs), dirs)
        }
        None => match anchor {
            // The directories to search on, filtered to only direct
            // descendants that start with `letter`.
            Some(letter) => {
                let dirs = dirs_with_anchor(letter, app.dirs.to_owned().unwrap());
                (search_string(&dirs), dirs)
            }
            None => (
                // All directories available to search on.
                app.search_string.to_owned().unwrap(),
                app.dirs.to_owned().unwrap(),
            ),
        },
    };

    // The fuzzy search command and options.
    let (fuzz_cmd, fuzz_theme, fuzz_nth) = if app.fuzzy_mode == Some(FuzzyMode::FZF) {
        ("fzf", FZF_THEME, "--with-nth=2..")
    } else {
        ("sk", SK_THEME, "--with-nth=3..")
    };

    // Print the directories to search on.
    let print = Command::new("printf")
        .arg(search_string)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("process should execute")
        .stdout
        .expect("failed to open print stdout");

    // Prepend line numbers to the output.
    let cat = Command::new("cat")
        .arg("-n")
        .stdin(Stdio::from(print))
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start cat")
        .stdout
        .expect("failed to open cat stdout");

    // Launch the relevant fuzzy utility to enable the directories
    // to be selected on. The line numbers are excluded from
    // being printed.
    let fuzz = Command::new(fuzz_cmd)
        .arg(fuzz_theme)
        .arg(fuzz_nth)
        .stdin(Stdio::from(cat))
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to start fzf")
        .stdout
        .expect("failed to open fuzz output");

    // Output the line number from the selection.
    let awk = Command::new("awk")
        .arg("{{print $1}}")
        .stdin(Stdio::from(fuzz))
        .stdout(Stdio::piped())
        .spawn()
        .expect("awk failed to start")
        .wait_with_output()
        .expect("failed to open awk stdout");

    let output = String::from_utf8(awk.stdout).expect("failed to open stdout");
    let trimmed = output.replace("\n", "");

    return match trimmed.parse::<usize>() {
        Ok(line_num) => {
            // On successful output we map the line number to the
            // corresponding path in `dirs` and return the path.
            let index = line_num - 1;
            dirs[index].path().into()
        }
        // If not successful return the initial path.
        Err(_) => app.path.to_owned(),
    };
}

pub fn clear_terminal() -> Result<ExitStatus, Error> {
    Command::new("cls")
        .status()
        .or_else(|_| Command::new("clear").status())
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

// Whether or not the entry is a direct descendant and starts with
// the case-insensitive `letter`.
fn starts_with(letter: String, entry: &DirEntry) -> bool {
    let binding = letter.to_uppercase();
    let (lowercase, uppercase) = (letter.as_str(), binding.as_str());

    entry.depth() == 1
        && entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with(lowercase) || s.starts_with(uppercase))
            .unwrap_or(false)
}

// Gets all the directories that satisfy the are direct descendants
// of `dirs` and start with the case-insensitive `letter`.
fn dirs_with_anchor(letter: String, dirs: Vec<DirEntry>) -> Vec<DirEntry> {
    dirs.into_iter()
        .filter(|e| starts_with(letter.clone(), &e))
        .collect()
}
