use std::path::PathBuf;

use anyhow::{anyhow, bail};
use clap::Parser;

use super::{config::Config, theme};
use crate::data::persistent_data;

type Color = cursive::theme::Color;

#[derive(Parser)]
#[command(
    author = "Tim Dubbins",
    about = "An audio player for the terminal with fuzzy-finder",
    version = "0.4.11"
)]
pub struct Args {
    /// The path to play or search on. Defaults to the current working directory
    #[arg(default_value = "")]
    pub path: PathBuf,

    /// Run an automated player without the TUI
    #[arg(short, long, default_value_t = false)]
    automate: bool,

    /// Set a default directory using the provided path
    #[arg(short, long, default_value_t = false)]
    set_default: bool,

    /// Run tap with the default directory, if set
    #[clap(short, long, default_value_t = 0, action = clap::ArgAction::Count)]
    // We use `u8` instead of `bool` so that this flag can be passed multiple
    // times. Defined as `false` if 0, `true` otherwise.
    default: u8,

    /// Print the default directory, if set
    #[arg(short, long, default_value_t = false)]
    print_default: bool,

    /// Exclude directories without audio
    #[arg(short, long, default_value_t = false)]
    exclude: bool,

    /// Use the terminal background color
    #[arg(short = 'b', long, default_value_t = false)]
    term_bg: bool,

    /// Use the terminal foreground and background colors only
    #[arg(short = 'c', long, default_value_t = false)]
    term_color: bool,

    /// Set the color scheme with <NAME>=<COLOR>
    /// For example:
    ///'--color fg=268bd2,bg=002b36,hl=fdf6e3,prompt=586e75,header=859900,header+=cb4b16,progress=6c71c4,info=2aa198,err=dc322f'
    #[arg(
        long,
        value_parser = Args::parse_color,
        value_delimiter = ',',
        verbatim_doc_comment,
    )]
    color: Vec<(String, Color)>,
}

impl Args {
    pub fn parse_args() -> Result<Args, anyhow::Error> {
        let mut args = Args::parse();
        args.validate()?;
        args.path = args.parse_path()?;
        Ok(args)
    }

    fn parse_path(&self) -> Result<PathBuf, anyhow::Error> {
        let path = if self.path.as_os_str().is_empty() {
            if self.automate {
                bail!("'--automate' requires a 'path' argument")
            } else if self.set_default {
                bail!("'--set-default' requires a 'path' argument")
            } else if self.default > 0 {
                persistent_data::cached_path()?
            } else {
                std::env::current_dir()?
            }
        } else if self.print_default {
            bail!("'--print-default' cannot be used with a 'path' argument")
        } else {
            self.path.clone()
        };

        if !path.exists() {
            bail!("'{}' doesn't exist", path.display())
        }

        Ok(path.canonicalize()?)
    }

    pub fn update_config_flags(&self, mut config: Config) -> Config {
        if self.term_bg {
            config.use_term_bg = true
        }
        if self.term_color {
            config.use_term_default = true
        }
        if self.exclude {
            config.exclude_non_audio = true
        }

        config
    }

    pub fn colors(&self) -> Vec<(String, Color)> {
        self.color.to_owned()
    }

    fn validate(&self) -> Result<(), anyhow::Error> {
        if self.automate && self.print_default {
            bail!("'--automate' cannot be used with '--print-default'")
        } else if self.automate && self.set_default {
            bail!("'--automate' cannot be used with '--set-default'")
        } else if self.print_default && self.set_default {
            bail!("'--print-default' cannot be used with '--set-default'")
        }
        Ok(())
    }

    fn parse_color(s: &str) -> Result<(String, Color), anyhow::Error> {
        let delimiter_pos = s.find('=').ok_or_else(|| {
            anyhow!(
                "{}invalid color argument: no '=' found in '{s}' for '--color <COLOR>'\n\n\
                for example, to set the foreground and background colors use:\n\n\
                '--color fg=<HEX>,bg=<HEX>'",
                format_stderr(s)
            )
        })?;

        let (name, value) = s.split_at(delimiter_pos);
        let (name, value) = (name.to_string(), value[1..].to_string());

        let color = Color::parse(&value).ok_or_else(|| {
            anyhow!(
                "{}Invalid color value '{}' for '--color <COLOR>'. 
                Example values: 'red', 'light green', '#123456'",
                format_stderr(s),
                value
            )
        })?;

        if !theme::validate_color(&name) {
            bail!(
                "{}Invalid color name '{}' for '--color <COLOR>'. 
                Available names are: 'fg', 'bg', 'hl', 'prompt', 'header_1', 'header_2 'progress', 'info', 'err'", 
                format_stderr(s),
                name);
        }

        Ok((name, color))
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    AutomatePlayer,
    PrintDefault,
    SetDefault,
    UseDefault,
    None,
}

impl Command {
    pub fn parse_command(args: &Args) -> Self {
        if args.automate {
            Self::AutomatePlayer
        } else if args.set_default {
            Self::SetDefault
        } else if args.print_default {
            Self::PrintDefault
        } else if args.default > 0 && args.path.as_os_str().is_empty() {
            Self::UseDefault
        } else {
            Self::None
        }
    }
}

pub fn search_root() -> PathBuf {
    // FIXME -- this is replaced by path prop of Config
    // parse_path().expect("should be verified on startup")
    PathBuf::new()
}

// Hack used to format error messages by overwriting clap stderr
// with 'spaces'.
fn format_stderr(s: &str) -> String {
    // There are 50 chars in the clap error message, excluding
    // the chars from user input.
    let spaces = s.len() + 50;
    format!("\r{: <1$}\r[tap error]: ", " ", spaces)
}
