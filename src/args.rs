use std::{fs::File, io::Write, path::PathBuf};

use anyhow::bail;
use clap::Parser;

use crate::utils::remove_trailing_slash;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    path: Option<PathBuf>,
    second_path: Option<PathBuf>,
}

impl Args {
    pub fn parse_path() -> Result<(PathBuf, String), anyhow::Error> {
        let path = match Args::parse().second_path {
            Some(p) => p,
            None => match Args::parse().path {
                Some(p) => p,
                None => std::env::current_dir()?,
            },
        };

        if !path.exists() {
            bail!("'{}' doesn't exist.", path.display())
        }

        // We remove trailing slashes from the path in order
        // to provide consistent behavior between OSs.
        let p = remove_trailing_slash(path)?;

        let s = match p.clone().into_os_string().into_string() {
            Ok(s) => s.replace(" ", r"\ "),
            Err(_) => bail!("Couldn't convert '{}' to string.", p.display()),
        };

        let mut file = File::create("foo.txt")?;
        file.write_all(s.as_bytes())?;
        Ok((p, s))
    }
}
