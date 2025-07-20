use std::path::PathBuf;

use {anyhow::bail, clap::Parser};

use crate::TapError;

// A struct that represents the command line arguments.
#[derive(Debug, Parser)]
#[command(
    author = "Tim Dubbins",
    about = "An audio player for the terminal with fuzzy-finder",
    version = crate::config::VERSION,
)]
pub struct Args {
    /// The path to play or search on.
    #[arg(index = 1)]
    pub path: Option<PathBuf>,

    /// Set a default directory using the provided path
    #[arg(short = 's', long = "set")]
    pub set_default_path: bool,

    /// Run tap with the default directory, if set
    #[arg(short = 'd', long = "default")]
    pub use_default_path: bool,

    /// Print the default directory, if set
    #[arg(short = 'p', long = "print")]
    pub print_default_path: bool,

    /// Use the terminal background color
    #[arg(short = 'b', long = "term_bg")]
    pub term_bg: bool,

    /// Use the terminal foreground and background colors only
    #[arg(short = 't', long = "term_color")]
    pub term_color: bool,

    /// Use the default color scheme
    #[arg(short = 'c', long = "default_color")]
    pub default_color: bool,

    /// Set the color scheme with <NAME>=<COLOR>
    /// For example:
    ///'--color fg=268bd2,bg=002b36,hl=fdf6e3,prompt=586e75,header_1=859900,header_2=cb4b16,progress=6c71c4,info=2aa198,err=dc322f'
    #[arg(long = "color", verbatim_doc_comment)]
    pub color: Option<String>,

    /// Run an audio player in the terminal without the TUI
    #[arg(long = "cli")]
    pub use_cli_player: bool,

    /// Print the current version
    #[arg(short = 'v', long = "version")]
    pub check_version: bool,
}

impl Args {
    pub fn parse_args() -> Result<Self, TapError> {
        let args = Self::try_parse()?;
        args.validate()?;

        Ok(args)
    }

    fn validate(&self) -> Result<(), TapError> {
        if self.print_default_path && self.path.is_some() {
            bail!("'--print' cannot be used with a 'path' argument");
        }

        if self.use_cli_player && self.path.is_none() {
            bail!("'--cli' requires a 'path' argument");
        }

        if self.set_default_path && self.path.is_none() {
            bail!("'--set' requires a 'path' argument");
        }

        if self.use_cli_player && self.print_default_path {
            bail!("'--cli' cannot be used with '--print'")
        }

        if self.use_cli_player && self.set_default_path {
            bail!("'--cli' cannot be used with '--set'")
        }

        if self.print_default_path && self.set_default_path {
            bail!("'--print' cannot be used with '--set'")
        }

        Ok(())
    }
}
