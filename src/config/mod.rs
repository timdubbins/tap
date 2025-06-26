pub mod file;
pub mod keybinding;
pub mod theme;

pub use self::{
    file::FileConfig,
    theme::{ColorStyles, Theme},
};

use std::{env, path::PathBuf};

use anyhow::{bail, Context};

use crate::{cli::Args, TapError};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Program-wide configuration. Derived from merging default values with
#[derive(Debug, Default)]
pub struct Config {
    pub check_version: bool,
    pub search_root: PathBuf,
    pub default_path: Option<PathBuf>,
    pub set_default_path: bool,
    pub use_default_path: bool,
    pub print_default_path: bool,
    pub use_cli_player: bool,
    pub theme: Theme,
    term_bg: bool,
    term_color: bool,
    default_color: bool,
}

impl Config {
    pub fn parse_config() -> Result<Self, TapError> {
        let mut config = Self::default();
        let file_config = FileConfig::deserialize().unwrap_or_default();
        let args = Args::parse_args()?;

        if args.check_version {
            config.check_version = true;

            return Ok(config);
        }

        config.parse_path(&file_config, &args)?;
        config.merge_flags(&file_config, &args);
        config.parse_colors(file_config, args)?;

        Ok(config)
    }

    fn parse_path(&mut self, file_config: &FileConfig, args: &Args) -> Result<(), TapError> {
        if args.use_default_path && file_config.path.is_none() {
            bail!("Default path not set");
        }

        let default_path = file_config.expanded_path();

        self.search_root = match args.path.as_ref().or(default_path.as_ref()) {
            Some(path) => path.clone(),
            None => env::current_dir().with_context(|| "Failed to get working directory")?,
        }
        .canonicalize()?;

        if !self.search_root.exists() {
            bail!("No such path: {:?}", self.search_root);
        }

        self.default_path = default_path;

        Ok(())
    }

    fn merge_flags(&mut self, file_config: &FileConfig, args: &Args) {
        // Update `self` with the config file settings.
        file_config.term_bg.map(|v| self.term_bg = v);
        file_config.term_color.map(|v| self.term_color = v);
        file_config.default_color.map(|v| self.default_color = v);

        // Update `self` with the command line args.
        self.set_default_path |= args.set_default_path;
        self.use_default_path |= args.use_default_path;
        self.print_default_path |= args.print_default_path;
        self.term_bg |= args.term_bg;
        self.term_color |= args.term_color;
        self.default_color |= args.default_color;
        self.use_cli_player |= args.use_cli_player;
    }

    fn parse_colors(&mut self, file_config: FileConfig, args: Args) -> Result<(), TapError> {
        let mut theme = Theme::default();

        if self.default_color {
            self.theme = theme;

            return Ok(());
        }

        let args_theme: Theme = args.color.unwrap_or_default().try_into()?;
        let file_theme: Theme = file_config.color.unwrap_or_default().into();
        let term_bg = self.term_bg && args_theme.get("bg").is_none();

        if self.term_color && args_theme.is_empty() {
            theme.set_term_color();
        } else {
            theme.extend(file_theme);
            theme.extend(args_theme);

            if term_bg {
                theme.set_term_bg();
            }
        }

        self.theme = theme;

        Ok(())
    }
}
