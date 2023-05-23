use std::env;
use std::path::PathBuf;

use crate::app::App;
use crate::utils::{env_var_includes, path_contains_dir};

const FZF_CMD: &'static str = "fzf --color bg+:#131415,bg:#131415,border:#b294bb,spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,info:#b294bb,pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,preview-bg:#D9D9D9,prompt:#616161,hl+:#b9ca4a";
const SK_CMD: &'static str = "sk --color dark,border:#b294bb,spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,info:#b294bb,pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,prompt:#616161,hl+:#b9ca4a";

#[derive(Clone, Copy, PartialEq)]
pub enum SearchMode {
    Fuzzy,
    NonFuzzy,
}

impl SearchMode {
    pub fn get_from(path: &PathBuf) -> Self {
        let fuzzy_available = env_var_includes(&["fzf"]) || env_var_includes(&["sk"]);
        match path_contains_dir(path) && fuzzy_available {
            true => SearchMode::Fuzzy,
            false => SearchMode::NonFuzzy,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SearchDir {
    CurrentDir,
    PathArg,
}

impl SearchDir {
    pub fn get_from(path: &PathBuf) -> Self {
        match *path == env::current_dir().unwrap() {
            true => SearchDir::CurrentDir,
            false => SearchDir::PathArg,
        }
    }
}

pub fn build_arg(app: &App) -> String {
    let current_exe = env::current_exe().unwrap();
    let fd_available = env_var_includes(&["fd"]);
    let fuzzy_query = match env_var_includes(&["fzf"]) {
        true => FZF_CMD,
        false => SK_CMD,
    };
    let res: String;

    match (app.search_dir, fd_available) {
        (SearchDir::CurrentDir, true) => {
                res = format!(
                    "{:?} \"$(fd -t d | {})\" --search-options 0",
                    current_exe,
                    fuzzy_query
                )
            }
            
        (SearchDir::CurrentDir, false) => {
                res = format!(
                    "{:?} ./\"$(find . -type d | sed -n 's|^./||p' | sort | {})\" --search-options 0",
                    current_exe,
                    fuzzy_query
                )
        }

        (SearchDir::PathArg, true) => {
                res = format!(
                    "{:?} {}/\"$(fd . \'{}\' -t d | sed -n 's|^{}/||p' | {})\" --search-options 1 --initial-path {}",
                    current_exe,
                    app.initial_path,
                    app.initial_path,
                    app.initial_path,
                    fuzzy_query,
                    app.initial_path
                )
            }

        (SearchDir::PathArg, false) => {
                res = format!(
                    "{:?} {}/\"$(find \'{}\' -type d | sed -n 's|^{}/||p' | sort | {})\" --search-options 1 --initial-path {}",
                    current_exe,
                    app.initial_path,
                    app.initial_path,
                    app.initial_path,
                    fuzzy_query,
                    app.initial_path,
                )
            }
        }

    res
}
