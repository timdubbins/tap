use std::{os::unix::prelude::OsStrExt, path::PathBuf};

use anyhow::bail;

pub fn env_var_includes(programs: &[&str]) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for sub_str in path.split(":") {
            for prog in programs {
                let p_str = format!("{}/{}", sub_str, prog);
                if std::fs::metadata(p_str).is_ok() {
                    return true;
                }
            }
        }
    }
    false
}

pub fn has_child_dir(path: &PathBuf) -> bool {
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

pub fn remove_trailing_slash(p: PathBuf) -> Result<PathBuf, anyhow::Error> {
    if has_trailing_slash(&p) {
        match p.clone().into_os_string().into_string() {
            Ok(s) => {
                let mut s = s;
                s.pop();
                Ok(PathBuf::from(s))
            }
            Err(_) => bail!("Couldn't remove trailing slash from {:?}", p),
        }
    } else {
        Ok(p)
    }
}

fn has_trailing_slash(p: &PathBuf) -> bool {
    p.as_os_str().as_bytes().last() == Some(&b'/')
}
