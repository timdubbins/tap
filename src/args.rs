use std::env;
use std::io::Error;
use std::path::PathBuf;

use clap::Parser;

use crate::search::{SearchDir, SearchMode};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    path: Option<PathBuf>,

    #[clap(hide(true), default_value = None, long)]
    mode: Option<u8>,

    #[clap(hide(true), default_value = None, long)]
    search_options: Option<u8>,

    #[clap[hide(true), default_value = None, long]]
    initial_path: Option<String>,
}

impl Args {
    pub fn parse_first_run() -> bool {
        match Args::parse().search_options {
            Some(_) => false,
            None => true,
        }
    }
    pub fn parse_path_args() -> Result<(PathBuf, String), Error> {
        let path = match Args::parse().path {
            Some(p) => p,
            None => env::current_dir()?,
        };

        let initial_path = match Args::parse().initial_path {
            Some(p) => p,
            None => path.clone().into_os_string().into_string().unwrap(),
        };

        Ok((path, initial_path))
    }

    pub fn parse_search_options(path: &PathBuf) -> (SearchMode, SearchDir) {
        match Args::parse().search_options {
            None => (SearchMode::get_from(path), SearchDir::get_from(path)),
            Some(0) => (SearchMode::NonFuzzy, SearchDir::CurrentDir),
            Some(1) => (SearchMode::NonFuzzy, SearchDir::PathArg),
            Some(2) => (SearchMode::Fuzzy, SearchDir::CurrentDir),
            Some(3) => (SearchMode::Fuzzy, SearchDir::PathArg),
            Some(4_u8..=u8::MAX) => panic!("invalid search options"),
        }
    }
}
