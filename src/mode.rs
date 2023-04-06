use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::args::Args;
use crate::utils::{env_var_includes, path_contains_dir};

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    NoFuzzyCurrentDir,
    NoFuzzyPathArg,
    FuzzyCurrentDir,
    FuzzyPathArg,
}

impl Mode {
    pub fn get_mode(path: &PathBuf) -> Self {
        if Mode::fuzzy_searchable(&path) {
            if *path == env::current_dir().unwrap() {
                Mode::FuzzyCurrentDir
            } else {
                Mode::FuzzyPathArg
            }
        } else {
            if *path == env::current_dir().unwrap() {
                Mode::NoFuzzyCurrentDir
            } else {
                Mode::NoFuzzyPathArg
            }
        }
    }

    pub fn restart_command(&self, path: PathBuf) {
        Command::new("/bin/bash")
            .arg("-c")
            .arg(self.arg_string(path))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    pub fn is_fuzzy_searchable(&self) -> bool {
        *self == Mode::FuzzyCurrentDir || *self == Mode::FuzzyPathArg
    }

    fn fuzzy_searchable(path: &PathBuf) -> bool {
        path_contains_dir(path) && env_var_includes("fzf") && env_var_includes("fd")
    }

    fn arg_string(&self, path: PathBuf) -> String {
        match self {
            Self::NoFuzzyCurrentDir => {
                format!("{} {}", env::current_exe().unwrap().display(), "--mode 0")
            }
            Self::NoFuzzyPathArg => format!(
                "{} \"{}\" {}",
                env::current_exe().unwrap().display(),
                path.into_os_string().into_string().unwrap(),
                "--mode 1"
            ),
            Self::FuzzyCurrentDir => format!(
                "{} {} {}",
                env::current_exe().unwrap().display(),
                "\"$(fd -t d | fzf)\"",
                "--mode 2",
            ),
            Self::FuzzyPathArg => {
                let initial_path = Args::get_path_arg();

                format!(
                    "{} {} \'{}\' {} {} {} {}",
                    env::current_exe().unwrap().display(),
                    "\"$(fd -t d .",
                    initial_path,
                    "| fzf)\"",
                    "--mode 3",
                    "--initial-path",
                    initial_path,
                )
            }
        }
    }
}
