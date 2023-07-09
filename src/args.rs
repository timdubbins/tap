use std::path::PathBuf;

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
    pub fn parse_path() -> Result<PathBuf, anyhow::Error> {
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

        Ok(p)
    }
}
