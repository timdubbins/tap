use std::io::Error;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use rand::Rng;

use crate::app::{App, SearchDir};
use crate::utils::env_var_includes;

// FZF prefix to set colors.
const FZF_CMD: &'static str = "fzf --color bg+:#131415,bg:#131415,border:#b294bb,spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,info:#b294bb,pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,preview-bg:#D9D9D9,prompt:#616161,hl+:#b9ca4a";

// Skim prefix to set colors.
const SK_CMD: &'static str = "sk --color dark,border:#b294bb,spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,info:#b294bb,pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,prompt:#616161,hl+:#b9ca4a";

pub fn get_fuzzy_cmd() -> String {
    if env_var_includes(&["fzf"]) {
        FZF_CMD.into()
    } else {
        SK_CMD.into()
    }
}

// Gets the number of subdirectories.
pub fn get_dir_count(app: &App) -> i32 {
    let arg = match (app.search_dir, app.fd_available) {
        (SearchDir::CurrentDir, true) => String::from("fd -t d --min-depth 1 | wc -l"),
        (SearchDir::CurrentDir, false) => {
            String::from(r"find . -type d -mindepth 1 \( -name '.?*' -prune -o -print \) | wc -l")
        }
        (SearchDir::PathArg, true) => {
            format!("fd . {} -t d --min-depth 1 | wc -l", app.path_string,)
        }
        (SearchDir::PathArg, false) => format!(
            r"find {} -type d -mindepth 1 \( -name '.?*' -prune -o -print \) | wc -l",
            app.path_string,
        ),
    };

    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg(arg)
        .output()
        .expect("path from random");

    let output_string = String::from_utf8(output.stdout).unwrap();
    let replaced = output_string.replace("\n", "");
    let trimmed = replaced.trim();

    trimmed.parse::<i32>().unwrap()
}

// Gets the path of a random subdirectory.
pub fn get_random_path(app: &App, dir_count: i32) -> PathBuf {
    let rand = rand::thread_rng().gen_range(1..dir_count);

    let arg = match (app.search_dir, app.fd_available) {
        (SearchDir::CurrentDir, true) => {
            format!("fd -t d --min-depth 1 --absolute-path | sed -n '{}p'", rand)
        }
        (SearchDir::CurrentDir, false) => format!(
            r"find . -type d -mindepth 1 \( -name '.?*' -prune -o -print \) | sed -n '{}p'",
            rand
        ),
        (SearchDir::PathArg, true) => format!(
            "fd . {} -t d --min-depth 1 | sed -n '{}p'",
            app.path_string, rand
        ),
        (SearchDir::PathArg, false) => format!(
            r"find {} -type d -mindepth 1 \( -name '.?*' -prune -o -print \) | sed -n '{}'p",
            app.path_string, rand
        ),
    };

    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg(arg)
        .output()
        .expect("path from random");

    let stdout = String::from_utf8(output.stdout).unwrap_or_default();

    PathBuf::from(stdout.replace("\n", ""))
}

// Gets the path of a subdirectory chosen via fuzzy selection.
pub fn get_fuzzy_path(app: &App) -> PathBuf {
    let arg = match (app.search_dir, app.fd_available) {
        (SearchDir::CurrentDir, true) => {
            format!("fd -t d --min-depth 1 | {}", app.fuzzy_cmd)
        }
        (SearchDir::CurrentDir, false) => format!(
            r"find . -type d -mindepth 1 \( -name '.?*' -prune -o -print \) | sed -n 's|^./||p' | sort | {}",
            app.fuzzy_cmd
        ),
        (SearchDir::PathArg, true) => format!(
            "fd . {} -t d --min-depth 1 | sed -n 's|^{}/||p' | {}",
            app.path_string, app.path_string, app.fuzzy_cmd
        ),
        (SearchDir::PathArg, false) => format!(
            "find {} -type d -mindepth 1 | sed -n 's|^{}/||p' | sort | {}",
            app.path_string, app.path_string, app.fuzzy_cmd
        ),
    };

    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg(arg)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let relative_path = PathBuf::from(stdout.replace("\n", ""));
    let mut path = app.path.clone();

    path.push(relative_path);
    path
}

pub fn clear_terminal() -> Result<ExitStatus, Error> {
    Command::new("cls")
        .status()
        .or_else(|_| Command::new("clear").status())
}