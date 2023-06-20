use std::env;
use std::io::Error;
use std::process::{Command, ExitStatus};

use crate::app::{App, SearchDir};
use crate::utils::env_var_includes;

// FZF prefix to set colors.
const FZF_CMD: &'static str = "fzf --color bg+:#131415,bg:#131415,border:#b294bb,spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,info:#b294bb,pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,preview-bg:#D9D9D9,prompt:#616161,hl+:#b9ca4a";

// Skim prefix to set colors.
const SK_CMD: &'static str = "sk --color dark,border:#b294bb,spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,info:#b294bb,pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,prompt:#616161,hl+:#b9ca4a";

// #[derive(Clone, Copy, PartialEq)]
// pub enum SearchMode {
//     Fuzzy,
//     NonFuzzy,
//     // Random,
// }

// impl SearchMode {
//     pub fn get_from(path: &PathBuf) -> Self {
//         let fuzzy_available = env_var_includes(&["fzf"]) || env_var_includes(&["sk"]);
//         match path_contains_dir(path) && fuzzy_available {
//             true => SearchMode::Fuzzy,
//             false => SearchMode::NonFuzzy,
//         }
//     }
// }

// #[derive(Clone, Copy, PartialEq)]
// pub enum SearchDir {
//     CurrentDir,
//     PathArg,
// }

// impl SearchDir {
//     pub fn get_from(path: &PathBuf) -> Self {
//         match *path == env::current_dir().unwrap() {
//             true => SearchDir::CurrentDir,
//             false => SearchDir::PathArg,
//         }
//     }
// }

// tested
pub fn get_dir_count(app: &App) -> i32 {
    let fd_available = env_var_includes(&["fd"]);

    let arg = match (app.search_dir, fd_available) {
        (SearchDir::CurrentDir, true) => String::from("fd -t d --min-depth 1 | wc -l"),
        (SearchDir::CurrentDir, false) => {
            String::from(r"find . -type d -mindepth 1 \( -name '.?*' -prune -o -print \) | wc -l")
        }
        (SearchDir::PathArg, true) => {
            format!("fd . {} -t d --min-depth 1 | wc -l", app.initial_path,)
        }
        (SearchDir::PathArg, false) => format!(
            r"find {} -type d -mindepth 1 \( -name '.?*' -prune -o -print \) | wc -l",
            app.initial_path,
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

// tested
pub fn get_path_string(app: &App, rand: i32) -> String {
    let fd_available = env_var_includes(&["fd"]);
    let rand = rand.to_string();

    let arg = match (app.search_dir, fd_available) {
        (SearchDir::CurrentDir, true) => {
            format!("fd -t d --min-depth 1 | sed -n '{}p'", rand)
        }
        (SearchDir::CurrentDir, false) => format!(
            r"find . -type d -mindepth 1 \( -name '.?*' -prune -o -print \) | sed -n '{}p'",
            rand
        ),
        (SearchDir::PathArg, true) => format!(
            "fd . {} -t d --min-depth 1 | sed -n '{}p'",
            app.initial_path, rand
        ),
        (SearchDir::PathArg, false) => format!(
            r"find {} -type d -mindepth 1 \( -name '.?*' -prune -o -print \) | sed -n '{}'p",
            app.initial_path, rand
        ),
    };

    let output = Command::new("/bin/bash")
        .arg("-c")
        .arg(arg)
        .output()
        .expect("path from random");

    let output = String::from_utf8(output.stdout).unwrap();

    // TODO - maybe there's a better way to sanitize the stdout
    let replaced = output
        .replace("\n", "")
        .replace(" ", r"\ ")
        .replace("'", r"\'")
        .replace("(", r"\(")
        .replace(")", r"\)")
        .replace("&", r"\&")
        .replace("$", r"\$")
        .replace("#", r"\#")
        .replace("?", r"\?")
        .replace("!", r"\!");

    replaced.trim().into()
}

pub fn restart_with_path_string(app: &App, path_string: String) {
    let current_exe = env::current_exe().unwrap();

    let arg = match app.search_dir {
        SearchDir::CurrentDir => {
            format!("{:?} {} --search-options 0", current_exe, path_string)
        }
        SearchDir::PathArg => format!(
            "{:?} {} --search-options 1 --initial-path {}",
            current_exe, path_string, app.initial_path
        ),
    };

    Command::new("/bin/bash")
        .arg("-c")
        .arg(arg)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

pub fn restart_with_fuzzy_query(app: &App) {
    let current_exe = env::current_exe().unwrap();
    let fd_available = env_var_includes(&["fd"]);

    let fuzzy_query = match env_var_includes(&["fzf"]) {
        true => FZF_CMD,
        false => SK_CMD,
    };

    let arg = match (app.search_dir, fd_available) {
        (SearchDir::CurrentDir, true) => format!(
            "{:?} \"$(fd -t d | {})\" --search-options 0",
            current_exe,
            fuzzy_query
        ),
        (SearchDir::CurrentDir, false) => format!(
            "{:?} ./\"$(find . -type d | sed -n 's|^./||p' | sort | {})\" --search-options 0",
            current_exe,
            fuzzy_query
        ),
        (SearchDir::PathArg, true) => format!(
            "{:?} {}/\"$(fd . \"{}\" -t d | sed -n 's|^{}/||p' | {})\" --search-options 1 --initial-path {}",
            current_exe,
            app.initial_path,
            app.initial_path,
            app.initial_path,
            fuzzy_query,
            app.initial_path
        ),
        (SearchDir::PathArg, false) => format!(
            "{:?} {}/\"$(find \"{}\" -type d | sed -n 's|^{}/||p' | sort | {})\" --search-options 1 --initial-path {}",
            current_exe,
            app.initial_path,
            app.initial_path,
            app.initial_path,
            fuzzy_query,
            app.initial_path,
        ),
    };

    Command::new("/bin/bash")
        .arg("-c")
        .arg(arg)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

pub fn clear_terminal() -> Result<ExitStatus, Error> {
    Command::new("cls")
        .status()
        .or_else(|_| Command::new("clear").status())
}
