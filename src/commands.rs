use std::io::Error;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use rand::Rng;

use crate::app::{App, FuzzyMode};

// Command to run fzf with the defined colors.
const FZF_CMD: &'static str = "fzf --color bg+:#131415,bg:#131415,border:#b294bb,spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,info:#b294bb,pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,preview-bg:#D9D9D9,prompt:#616161,hl+:#b9ca4a";

// Command to run sk with the defined colors.
const SK_CMD: &'static str = "sk --color dark,border:#b294bb,spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,info:#b294bb,pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,prompt:#616161,hl+:#b9ca4a";

// Gets the number of subdirectories.
pub fn get_dir_count(app: &App) -> i32 {
    // Command to list all child directories, excluding hidden directories.
    let find_dirs = find_dirs(app.fd_available, false);

    // Command to count number of lines.
    let line_count: &'static str = "wc -l";

    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg(format!("{} | {}", find_dirs, line_count))
        .current_dir(&app.path)
        .output()
        .expect("process should execute");

    let output_string = String::from_utf8(output.stdout).unwrap();
    let replaced = output_string.replace("\n", "");
    let trimmed = replaced.trim();

    trimmed.parse::<i32>().expect("should be a numeric string")
}

// Gets the path of a random subdirectory.
pub fn get_random_path(app: &App, dir_count: i32) -> PathBuf {
    // A random number in range [1...`number of child directories`].
    let line = rand::thread_rng().gen_range(1..dir_count);

    // Command to print the absolute paths of all child directories,
    // excluding hidden directories.
    let find_dirs = match app.fd_available {
        true => format!("fd -t d --min-depth 1 --absolute-path"),
        false => format!(r"find ~+ -mindepth 1 -type d \( -name '.?*' -prune -o -print \)"),
    };

    // Command to restrict lines printed to line number `line`.
    let print_line = format!("sed -n '{}p'", line);

    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg(format!("{} | {}", find_dirs, print_line))
        .current_dir(&app.path)
        .output()
        .expect("process should execute");

    let stdout = String::from_utf8(output.stdout).unwrap();

    PathBuf::from(stdout.replace("\n", ""))
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
