use std::path::PathBuf;

use anyhow::bail;
use clap::{ArgGroup, Parser};

use crate::serde::get_cached;
#[derive(Parser)]
#[command(
    author = "Tim Dubbins",
    about = "an audio player for the terminal with fuzzy-finder",
    version = "0.4.4"
)]
#[clap(group = ArgGroup::new("exclude_multiples").multiple(false))]
#[clap(group = ArgGroup::new("conflicts_path").conflicts_with("path"))]
#[clap(group = ArgGroup::new("requires_path").requires("path"))]
pub struct Args {
    path: Option<PathBuf>,

    second_path: Option<PathBuf>,

    #[arg(short, long, default_value_t = false, group = "exclude_multiples")]
    automate: bool,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "exclude_multiples",
        group = "requires_path"
    )]
    set_default: bool,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "exclude_multiples",
        group = "conflicts_path"
    )]
    default: bool,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "exclude_multiples",
        group = "conflicts_path"
    )]
    print_default: bool,
}

impl Args {
    pub fn parse_path() -> Result<PathBuf, anyhow::Error> {
        let path = match Args::is_default() {
            true => get_cached::<PathBuf>("path")?,
            false => match Args::parse().second_path {
                Some(p) => p,
                None => match Args::parse().path {
                    Some(p) => p,
                    None => std::env::current_dir()?,
                },
            },
        };

        if !path.exists() {
            bail!("'{}' doesn't exist.", path.display())
        }

        Ok(path.canonicalize()?)
    }

    pub fn is_automated() -> bool {
        Args::parse().automate
    }

    pub fn is_default() -> bool {
        Args::parse().default
    }

    pub fn to_set_default() -> bool {
        Args::parse().set_default
    }

    pub fn to_print_default() -> bool {
        Args::parse().print_default
    }
}
