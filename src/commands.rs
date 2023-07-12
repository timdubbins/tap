use std::io::Error;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use rand::Rng;
use walkdir::{DirEntry, WalkDir};

use crate::app::{App, FuzzyMode};

// Command to run fzf with the defined colors.
const FZF_CMD: &'static str = "fzf --color bg+:#131415,bg:#131415,border:#b294bb,spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,info:#b294bb,pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,preview-bg:#D9D9D9,prompt:#616161,hl+:#b9ca4a";

// Command to run sk with the defined colors.
const SK_CMD: &'static str = "sk --color dark,border:#b294bb,spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,info:#b294bb,pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,prompt:#616161,hl+:#b9ca4a";

fn is_non_hidden_dir(entry: &walkdir::DirEntry) -> bool {
    is_dir(entry) && !is_hidden(entry)
}

// Returns true if the entry is a directory.
fn is_dir(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
}

// Returns true if the entry is hidden.
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

// Returns an array of all child directories, relative to `path`,
// excluding hidden directories.
pub fn get_entries(path: &PathBuf) -> Vec<DirEntry> {
    WalkDir::new(path)
        .min_depth(1)
        .into_iter()
        .filter_entry(is_non_hidden_dir)
        .map(Result::unwrap)
        .collect()
}

// Gets the path of a random subdirectory.
pub fn get_random_path(app: &App) -> PathBuf {
    let entries = app.entries.as_ref().unwrap();
    let target = rand::thread_rng().gen_range(0..entries.len() - 1);

    entries[target].to_owned().into_path()
}

pub fn get_string(entries: &Vec<DirEntry>) -> String {
    entries
        .into_iter()
        .map(|e| {
            e.file_name()
                .to_os_string()
                .into_string()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn _get_fuzzy_path(app: &App, second_path: Option<PathBuf>, anchor: Option<String>) -> PathBuf {
    let string = app.entries_string.as_ref().unwrap();
    let process = Command::new("/bin/bash")
        .arg("-c")
        .arg(format!(
            "printf \"{}\" | cat -n | fzf --with-nth 2.. | awk '{{print $1}}'",
            string
        ))
        .current_dir(&app.path)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("process should execute")
        .wait_with_output()
        .expect("wait should succeed on child");

    let output = String::from_utf8(process.stdout).unwrap();
    let trimmed = output.replace("\n", "");
    let index = trimmed
        .parse::<usize>()
        .expect("should be a numeric string")
        - 1;
    let entries = app.entries.to_owned().unwrap();

    entries[index].path().into()
}

// Gets the path of a subdirectory chosen via fuzzy selection.
pub fn get_fuzzy_path(app: &App, second_path: Option<PathBuf>, anchor: Option<String>) -> PathBuf {
    // The directory to use as the search root.
    let search_dir = match second_path {
        Some(p) => p,
        None => app.path.clone(),
    };

    // The list of directories to fuzzy search on.
    let find_dirs = match anchor {
        Some(anchor) => find_dirs_with_prefix(anchor, app.fd_available),
        None => find_dirs(app.fd_available, true),
    };

    // The available fuzzy command.
    let fuzzy_cmd = if app.fuzzy_mode == Some(FuzzyMode::FZF) {
        FZF_CMD
    } else {
        SK_CMD
    };

    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg(format!("{} | {}", find_dirs, fuzzy_cmd))
        .current_dir(&search_dir)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("process should execute")
        .wait_with_output()
        .expect("wait should succeed on child");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let component = PathBuf::from(stdout.replace("\n", ""));
    let mut path = PathBuf::from(search_dir);

    path.push(component);
    path
}

// Command to return a sorted list of all child directories, excluding
// hidden directories, that start with the letter `anchor`.
fn find_dirs_with_prefix(anchor: String, fd_available: bool) -> String {
    match fd_available {
        true => format!("fd '^{}' -t d --max-depth 1 | sort", anchor),
        false => format!(
            r"find . -maxdepth 1 -name '[{}{}]*' -type d \( -name '.?*' -prune -o -print \) \
            | sed -n 's|^./||p' | sort",
            anchor,
            anchor.to_uppercase(),
        ),
    }
}

// Command to list all child directories, excluding hidden directories.
fn find_dirs(fd_available: bool, is_printed: bool) -> String {
    // Command to remove the './' prefix from each directory.
    let formatting = match is_printed {
        true => " | sed -n 's|^./||p'",
        false => "",
    };

    match fd_available {
        true => format!("fd -t d --min-depth 1"),
        false => format!(
            r"find . -mindepth 1 -type d \( -name '.?*' -prune -o -print \){}",
            formatting,
        ),
    }
}

pub fn clear_terminal() -> Result<ExitStatus, Error> {
    Command::new("cls")
        .status()
        .or_else(|_| Command::new("clear").status())
}
