use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::bail;

pub fn env_var_includes(programs: &[&str]) -> bool {
    if let Ok(path) = env::var("PATH") {
        for sub_str in path.split(":") {
            for prog in programs {
                let p_str = format!("{}/{}", sub_str, prog);
                if fs::metadata(p_str).is_ok() {
                    return true;
                }
            }
        }
    }
    false
}

pub fn path_contains_dir(path: &PathBuf) -> bool {
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

pub fn path_to_string<P: Into<PathBuf>>(path: P) -> Result<String, anyhow::Error> {
    match path.into().clone().into_os_string().into_string() {
        Ok(p) => Ok(remove_trailing_slash(p)),
        Err(_) => bail!("Could not convert path to string"),
    }
}

fn remove_trailing_slash(mut s: String) -> String {
    if s.ends_with('/') {
        s.pop();
    }
    s
}
