use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::args::Args;

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

    fn fuzzy_searchable(path: &PathBuf) -> bool {
        Mode::path_contains_dir(path) && Mode::is_in_path("fzf") && Mode::is_in_path("fd")
    }

    fn is_in_path(program: &str) -> bool {
        if let Ok(path) = env::var("PATH") {
            for p in path.split(":") {
                let p_str = format!("{}/{}", p, program);
                if fs::metadata(p_str).is_ok() {
                    return true;
                }
            }
        }
        false
    }

    fn path_contains_dir(path: &PathBuf) -> bool {
        if path.is_dir() {
            for entry in path.read_dir().expect("directory should not be empty") {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        return true;
                    }
                }
            }
        } else {
            return false;
        }

        false
    }
}
