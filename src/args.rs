use std::io::Error;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    path: Option<PathBuf>,
}

impl Args {
    pub fn parse_args() -> Result<PathBuf, Error> {
        let path = match Args::parse().path {
            Some(p) => p,
            None => std::env::current_dir()?,
        };

        Ok(path)
    }
}
