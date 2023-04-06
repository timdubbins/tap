use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, PartialEq)]
pub enum Mode {
    NoFuzzy,
    FuzzyFromCurrentDir,
    FuzzyFromPathArg,
    // TODO: implement NoFuzzyFromPathArg
}

impl Mode {
    pub fn get_mode(path: &PathBuf) -> Self {
        if Mode::fuzzy_searchable(&path) {
            if *path == env::current_dir().unwrap() {
                Mode::FuzzyFromCurrentDir
            } else {
                Mode::FuzzyFromPathArg
            }
        } else {
            Mode::NoFuzzy
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
            Self::NoFuzzy => format!(
                "{} {}",
                env::current_exe().unwrap().display(),
                "--command-arg 0"
            ),
            Self::FuzzyFromCurrentDir => format!(
                "{} {} {}",
                env::current_exe().unwrap().display(),
                "\"$(fd -t d | fzf)\"",
                "--command-arg 1",
            ),
            Self::FuzzyFromPathArg => format!(
                "{} {} {} {} {}",
                env::current_exe().unwrap().display(),
                "\"$(fd -t d .",
                path.into_os_string().into_string().unwrap(),
                "| fzf)\"",
                "--command-arg 2",
            ),
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
