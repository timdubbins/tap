use std::path::PathBuf;

use anyhow::bail;
use clap::{ArgGroup, Parser};

use crate::serialization::get_cached;
use crate::theme::{COLOR_KEYS, UserColors};

type Color = cursive::theme::Color;

#[derive(PartialEq)]
pub enum Opts {
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
    version = "0.4.7"
)]
#[clap(group = ArgGroup::new("exclude_multiples").multiple(false))]
#[clap(group = ArgGroup::new("conflicts_path").conflicts_with("path"))]
#[clap(group = ArgGroup::new("requires_path").requires("path"))]
pub struct Args {
    #[arg(help = "The path to play or search on. Defaults to the current working directory")]
    path: Option<PathBuf>,

    #[arg(help = "Providing a second path overrides the first path")]
    second_path: Option<PathBuf>,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "exclude_multiples",
        help = "Run an automated player without the TUI"
    )]
    automate: bool,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "exclude_multiples",
        group = "requires_path",
        help = "Set a default directory using the provided path"
    )]
    set_default: bool,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "exclude_multiples",
        group = "conflicts_path",
        help = "Run tap with the default directory, if set"
    )]
    default: bool,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "exclude_multiples",
        group = "conflicts_path",
        help = "Print the default directory, if set"
    )]
    print_default: bool,

    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Use the terminal background color"
    )]
    term_bg: bool,

    #[arg(
        short, 
        long, 
        value_parser = parse_color, 
        value_delimiter = ',', 
        help = "Set custom colors. For example: '--color bg=<hex-color>,hl=<hex-color>'. Available fields: 'bg, hl, artist, album, track, status, prompt, bar, stop"
    )]
    colors: Vec<(String, Color)>,
}

pub fn parse_args() -> Result<(PathBuf, Opts), anyhow::Error> {
    let args = Args::parse();
    let option = parse_opts(&args);
    let path = parse_path(&args)?;

    Ok((path, option))
}

pub fn parse_user_colors() -> UserColors {
    let args = Args::parse();
    UserColors::new(args.colors, args.term_bg)
}

pub fn search_root() -> PathBuf {
    parse_args().expect("should be verified on startup").0
}

fn parse_path(args: &Args) -> Result<PathBuf, anyhow::Error> {
    let path = match args.default {
        true => get_cached::<PathBuf>("path")?,
        false => match &args.second_path {
            Some(p) => p.to_owned(),
            None => match &args.path {
                Some(p) => p.to_owned(),
                None => std::env::current_dir()?,
            },
        },
    };

    if !path.exists() {
        bail!("'{}' doesn't exist", path.display())
    }

    Ok(path.canonicalize()?)
}

fn parse_opts(args: &Args) -> Opts {
    if args.automate {
        Opts::Automate
    } else if args.default {
        Opts::Default
    } else if args.set_default {
        Opts::Set
    } else if args.print_default {
        Opts::Print
    } else {
        Opts::None
    }
}

fn parse_color(s: &str) -> Result<(String, Color), anyhow::Error> {
        let pos = match s.find('=') {
            Some(pos) => pos,
            None => bail!("invalid color argument: no `=` found in `{s}`"),
        };

    let (key, val): (String, Color) = (s[..pos].parse()?, (s[pos + 1..]).parse()?);

    for color_key in COLOR_KEYS {
        if key.eq(color_key) {
            return Ok((key, val));
        }
    }

    bail!("invalid color definition: no such definition `{key}`");
}
