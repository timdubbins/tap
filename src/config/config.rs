// TODO - we want a Config struct to handle all args, config file, keybindings, theme etc.
// this to replace creating lazy statics for args and palette.
// could also move persistent_data.rs logic to Config module, and session_data.rs to player module.

use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, path::PathBuf};

use cursive::theme::Color;

use super::{
    args::{Args, Command},
    theme,
};

// The struct used to deserialize our config file (`tap.yml`)
#[derive(Debug)]
pub struct Config {
    pub path: PathBuf,
    pub colors: HashMap<String, Color>,
    pub command: Command,
    pub use_term_bg: bool,
    pub use_term_default: bool,
    pub exclude_non_audio: bool,
}

impl Config {
    pub fn parse_config(args: Args) -> Self {
        let mut config = Self::from_file().unwrap_or_default();
        config.command = Command::parse_command(&args);
        config = args.update_config_flags(config);
        config = theme::parse_colors(args.colors(), config);
        config.path = args.path;
        config
    }

    fn from_file() -> Result<Self, anyhow::Error> {
        let home_dir = std::env::var("HOME")?;
        let config_path = std::path::PathBuf::from(home_dir).join(".tap.yml");
        let mut file = std::fs::File::open(config_path)?;
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut file, &mut contents)?;

        let config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            colors: HashMap::new(),
            command: Command::None,
            use_term_bg: false,
            use_term_default: false,
            exclude_non_audio: false,
        }
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            path: Option<PathBuf>,
            colors: Option<HashMap<String, String>>,
            use_term_bg: Option<bool>,
            use_term_default: Option<bool>,
            exclude_non_audio: Option<bool>,
        }

        let helper = Helper::deserialize(deserializer)?;

        let colors = helper
            .colors
            .unwrap_or(HashMap::new())
            .into_iter()
            .filter_map(|(name, value)| {
                if theme::validate_color(&name) {
                    Color::parse(&value).map(|color| (name, color))
                } else {
                    None
                }
            })
            .collect();

        Ok(Config {
            path: helper.path.unwrap_or(PathBuf::new()),
            colors,
            command: Command::None,
            use_term_bg: helper.use_term_bg.unwrap_or(false),
            use_term_default: helper.use_term_default.unwrap_or(false),
            exclude_non_audio: helper.exclude_non_audio.unwrap_or(false),
        })
    }
}
