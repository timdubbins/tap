use std::path::PathBuf;

use anyhow::bail;
use clap::{ArgGroup, Parser};

use crate::serde::get_cached;

#[derive(PartialEq)]
pub enum Options {
    Automate,
    Print,
    Set,
    Default,
    None,
}

#[derive(Parser)]
#[command(
    author = "Tim Dubbins",
    about = "An audio player for the terminal with fuzzy-finder",
    version = "0.4.4"
)]
#[clap(group = ArgGroup::new("exclude_multiples").multiple(false))]
#[clap(group = ArgGroup::new("conflicts_path").conflicts_with("path"))]
#[clap(group = ArgGroup::new("requires_path").requires("path"))]
pub struct Args {
    #[arg(help = "The path to play or search on. If omitted the current directory is used")]
    path: Option<PathBuf>,

    #[arg(help = "Providing a second path overrides the first path")]
    second_path: Option<PathBuf>,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "exclude_multiples",
        help = "Runs an automated player without the TUI"
    )]
    automate: bool,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "exclude_multiples",
        group = "requires_path",
        help = "Sets a default directory using the provided path"
    )]
    set_default: bool,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "exclude_multiples",
        group = "conflicts_path",
        help = "Runs tap with the default directory, if set"
    )]
    default: bool,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "exclude_multiples",
        group = "conflicts_path",
        help = "Prints the default directory, if set"
    )]
    print_default: bool,
}

impl Args {
    pub fn parse_args() -> Result<(PathBuf, Options), anyhow::Error> {
        let args = Args::parse();
        let option = Args::parse_options(&args);
        let path = Args::parse_path(args)?;

        Ok((path, option))
    }

    pub fn parse_path(args: Args) -> Result<PathBuf, anyhow::Error> {
        let path = match args.default {
            true => get_cached::<PathBuf>("path")?,
            false => match args.second_path {
                Some(p) => p,
                None => match args.path {
                    Some(p) => p,
                    None => std::env::current_dir()?,
                },
            },
        };

        if !path.exists() {
            bail!("'{}' doesn't exist", path.display())
        }

        Ok(path.canonicalize()?)
    }

    fn parse_options(args: &Args) -> Options {
        if args.automate {
            Options::Automate
        } else if args.default {
            Options::Default
        } else if args.set_default {
            Options::Set
        } else if args.print_default {
            Options::Print
        } else {
            Options::None
        }
    }
}
