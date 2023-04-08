use std::env;
use std::path::PathBuf;

use crate::app::App;
use crate::utils::{env_var_includes, path_contains_dir};

#[derive(Clone, Copy, PartialEq)]
pub enum SearchMode {
    Fuzzy,
    NonFuzzy,
}

impl SearchMode {
    pub fn get_from(path: &PathBuf) -> Self {
        match path_contains_dir(path) && env_var_includes(&["fzf", "fd"]) {
            true => SearchMode::Fuzzy,
            false => SearchMode::NonFuzzy,
        }
    }
}

#[derive(Clone, Copy)]
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

pub fn search_arg(app: &App) -> String {
    match (app.search_mode, app.search_dir) {
        (SearchMode::NonFuzzy, SearchDir::CurrentDir) => {
            format!(
                "{} {}",
                env::current_exe().unwrap().display(),
                "--search-options 0"
            )
        }
        (SearchMode::NonFuzzy, SearchDir::PathArg) => format!(
            "{} \"{}\" {}",
            env::current_exe().unwrap().display(),
            app.path.clone().into_os_string().into_string().unwrap(),
            "--search-options 1"
        ),
        (SearchMode::Fuzzy, SearchDir::CurrentDir) => format!(
            "{} {} {}",
            env::current_exe().unwrap().display(),
            "\"$(fd -t d | fzf)\"",
            "--search-options 2",
        ),
        (SearchMode::Fuzzy, SearchDir::PathArg) => format!(
            "{} {} \'{}\' {} {} {} {}",
            env::current_exe().unwrap().display(),
            "\"$(fd -t d .",
            app.initial_path,
            "| fzf)\"",
            "--search-options 3",
            "--initial-path",
            app.initial_path,
        ),
    }
}
