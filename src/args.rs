use std::path::PathBuf;

use anyhow::bail;
use clap::Parser;

use crate::serialization::get_cached;
use crate::theme;

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
    version = "0.4.8"
)]
pub struct Args {
    /// The path to play or search on. Defaults to the current working directory
    path: Option<PathBuf>,

    /// Run an automated player without the TUI
    #[arg( short, long, default_value_t = false)]
    automate: bool,

    /// Set a default directory using the provided path
    #[arg( short, long, default_value_t = false)]
    set_default: bool,

    /// Run tap with the default directory, if set
    #[clap( short, long, default_value_t = 0, action = clap::ArgAction::Count)]
    // We use `u8` instead of `bool` so that this flag can be passed multiple 
    // times. Defined as `false` if 0, `true` otherwise.
    default: u8,

    /// Print the default directory, if set
    #[arg( short, long, default_value_t = false)]
    print_default: bool,

    /// Use the terminal background color
    #[arg( short, long, default_value_t = false)]
    term_bg: bool,

    /// Set custom colors using <NAME>=<HEX>
    /// For example: 
    ///'--colors fg=268bd2,bg=002b36,hl=fdf6e3,prompt=586e75,header=859900,header+=cb4b16,progress=6c71c4,info=2aa198,err=dc322f'
    #[arg(
        short,
        long, 
        value_parser = parse_color, 
        value_delimiter = ',',
        verbatim_doc_comment,
    )]
    colors: Vec<(String, Color)>,
}

pub fn parse() -> Result<(PathBuf, Opts), anyhow::Error> {
    Ok((parse_path()?, parse_opts()?))
}

pub fn user_colors() -> (Vec<(String, Color)>, bool) {
    (ARGS.colors.to_owned(), ARGS.term_bg)
}

pub fn search_root() -> PathBuf {
    parse_path().expect("should be verified on startup")
}

fn parse_path() -> Result<PathBuf, anyhow::Error> {
    let path = match &ARGS.path {
        Some(p) => p.to_owned(),
        None => match ARGS.default > 0 {
            true => get_cached::<PathBuf>("path")?,
            false => std::env::current_dir()?,
        }
    };

    if !path.exists() {
        bail!("'{}' doesn't exist", path.display())
    }

    Ok(path.canonicalize()?)
}



fn parse_color(s: &str) -> Result<(String, Color), anyhow::Error> {
    let pos = match s.find('=') {
        Some(pos) => pos,
        None => bail!(
            "{}invalid color argument: no '=' found in '{s}' for '--colors <COLORS>'\n\n\
            for example, to set the foreground and background colors use:\n\n\
            '--colors fg=<HEX>,bg=<HEX>'", 
            format_stderr(s)
        ),
    };

    let (name, color): (String, String) = (s[..pos].parse()?, (s[pos + 1..]).parse()?);

    let hex: Color = match is_valid_hex_string(&color) && color.len() == 6 {
        true => color.parse()?,
        false => bail!(
            "{}invalid hex value '{color}' for '--colors <COLORS>'\n\n\
            valid values are in range '000000' -> 'ffffff'",
            format_stderr(s),
        ),
    };

    match theme::COLOR_MAP.contains_key(&name) {
        true => Ok((name, hex)),
        false => bail!(
            "{}invalid color name '{name}' for '--colors <COLORS>'\n\n\
            available names:\n\
            'fg', 'bg', 'hl', 'prompt', 'header', 'header+', 'progress', 'info', 'err'",
            format_stderr(s),
        ),
    }
}

fn parse_opts() -> Result<Opts, anyhow::Error> {
    exclude_multiple()?;
    conflicts_path()?;
    
    if ARGS.automate {
        Ok(Opts::Automate)
    } else if ARGS.set_default {
        Ok(Opts::Set)
    } else if ARGS.print_default {
        Ok(Opts::Print)
    } else if ARGS.default > 0 && ARGS.path.is_none() {
        Ok(Opts::Default)
    } else {
        Ok(Opts::None)
    }
}

fn exclude_multiple() -> Result<(), anyhow::Error> {
    if ARGS.automate && ARGS.print_default {
        bail!("'--automate' cannot be used with '--print-default'")
    } else if ARGS.automate && ARGS.set_default {
        bail!("'--automate' cannot be used with '--set-default'")
    } else if ARGS.print_default && ARGS.set_default {
        bail!("'--print-default' cannot be used with '--set-default'")
    }

    Ok(())
}

fn conflicts_path() -> Result<(), anyhow::Error> {
    if ARGS.automate && ARGS.path.is_none() {
            bail!("'--automate' requires a 'path' argument")
    } else if ARGS.set_default && ARGS.path.is_none() {
            bail!("'--set-default' requires a 'path' argument")
    } else if ARGS.print_default && ARGS.path.is_some() {
            bail!("'--print-default' cannot be used with a 'path' argument")
    }

    Ok(())
}

fn is_valid_hex_string(s: &str) -> bool {
    for c in s.chars() {
        if !c.is_digit(16)  {
            return false;
        }
    }
    true
}

// Hack used to format error messages by overwriting clap stderr
// with 'spaces'.
fn format_stderr(s: &str) -> String {
    // There are 50 chars in the clap error message, excluding
    // the chars from user input.
    let spaces = s.len() + 50;
    format!("\r{: <1$}\r[tap error]: ", " ", spaces)
}