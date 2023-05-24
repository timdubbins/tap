use std::env;
use std::path::PathBuf;

use anyhow::bail;
use clap::Parser;

use crate::search::{SearchDir, SearchMode};
use crate::utils::path_to_string;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    path: Option<PathBuf>,

    second_path: Option<PathBuf>,

    #[clap(hide(true), default_value = None, long)]
    search_options: Option<u8>,

    #[clap[hide(true), default_value = None, long]]
    initial_path: Option<String>,
}

impl Args {
    pub fn is_first_run() -> bool {
        match Args::parse().search_options {
            Some(_) => false,
            None => true,
        }
    }

    pub fn parse_path() -> Result<PathBuf, anyhow::Error> {
        let path = match Args::parse().second_path {
            Some(p) => p,
            None => match Args::parse().path {
                Some(p) => p,
                None => env::current_dir()?,
            },
        };

        if !path.exists() {
            bail!("{:?} doesn't exist.", path)
        }

        Ok(path)
    }

    fn get_containing_dir(path: &PathBuf) -> Result<PathBuf, anyhow::Error> {
        let parent = match path.parent() {
            Some(p) => p,
            None => bail!("{:?} doesn't have a `parent` folder.", path),
        };

        match path.is_dir() {
            true => Ok(parent.into()),
            false => match parent.parent() {
                Some(p) => Ok(p.into()),
                None => bail!("{:?} doesn't have a `grand-parent` folder.", path),
            },
        }
    }

    pub fn parse_initial_path(path: &PathBuf, fuzzy: bool) -> Result<String, anyhow::Error> {
        match Args::parse().initial_path {
            Some(p) => Ok(p),
            None => match fuzzy {
                true => Ok(path_to_string(path)?),
                false => {
                    let parent = Args::get_containing_dir(path)?;
                    Ok(path_to_string(parent)?)
                }
            },
        }
    }

    pub fn parse_search_options(path: &PathBuf) -> Result<(SearchMode, SearchDir), anyhow::Error> {
        match Args::parse().search_options {
            None => Ok((SearchMode::get_from(path), SearchDir::get_from(path))),
            Some(0) => Ok((SearchMode::Fuzzy, SearchDir::CurrentDir)),
            Some(1) => Ok((SearchMode::Fuzzy, SearchDir::PathArg)),
            _ => bail!("Invalid search options."),
        }
    }
}
