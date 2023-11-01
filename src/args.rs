use std::path::PathBuf;

use anyhow::bail;
use clap::{ArgGroup, Parser};

use crate::serialization::get_cached;
use crate::theme::COLOR_MAP;

type Color = cursive::theme::Color;

lazy_static::lazy_static! {
    static ref ARGS: Args = Args::parse();
}

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
        help = "Set custom colors using <color-key>=<hex-value>. 
        For example:
            '--colors fg=268bd2,bg=002b36,hl=fdf6e3,prompt=586e75,header=859900,header+=cb4b16,progress=6c71c4,info=2aa198,err=dc322f'"
    )]
    colors: Vec<(String, Color)>,
}

pub fn parse() -> Result<(PathBuf, Opts), anyhow::Error> {
    Ok((parse_path()?, parse_opts()))
}

pub fn user_colors() -> (Vec<(String, Color)>, bool) {
    (ARGS.colors.to_owned(), ARGS.term_bg)
}

pub fn search_root() -> PathBuf {
    parse_path().expect("should be verified on startup")
}

fn parse_path() -> Result<PathBuf, anyhow::Error> {
    let path = match ARGS.default {
        true => get_cached::<PathBuf>("path")?,
        false => match &ARGS.second_path {
            Some(p) => p.to_owned(),
            None => match &ARGS.path {
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

fn parse_opts() -> Opts {
    if ARGS.automate {
        Opts::Automate
    } else if ARGS.default {
        Opts::Default
    } else if ARGS.set_default {
        Opts::Set
    } else if ARGS.print_default {
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

    for color_key in COLOR_MAP.keys() {
        if key.eq(color_key) {
            return Ok((key, val));
        }
    }

    bail!("invalid color definition: no such definition `{key}`");
}
