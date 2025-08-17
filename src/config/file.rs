use std::{
    collections::HashMap,
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use {anyhow::bail, regex::Captures, regex::Regex, serde::Deserialize};

use crate::TapError;

// A struct that represents our `tap.yml` config file.
#[derive(Default, Deserialize)]
pub struct FileConfig {
    pub path: Option<PathBuf>,
    pub sequential: Option<bool>,
    pub color: Option<HashMap<String, String>>,
    pub term_bg: Option<bool>,
    pub term_color: Option<bool>,
    pub default_color: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct KeybindingOnly {
    keybinding: Option<HashMap<String, Vec<String>>>,
}

impl FileConfig {
    pub fn update_path(path: &PathBuf) -> Result<(), TapError> {
        let file_path = match Self::find() {
            Ok(p) => p,
            Err(_) => return Ok(()),
        };

        let config_content = match fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(_) => return Ok(()),
        };

        // Regex that matches:
        //  - the prefix (whitespace, optional '#' and "path:" plus following whitespace)
        //  - then either a double-quoted string or a single-quoted string.
        let re = Regex::new(
            r#"(?m)^(?P<prefix>\s*#?\s*path:\s*)(?:"(?P<value_d>[^"]*)"|'(?P<value_s>[^']*)')"#,
        )?;

        let updated_content = if re.is_match(&config_content) {
            re.replace_all(&config_content, |caps: &Captures| {
                let standardized_prefix = "path: ";
                let quote = if caps.name("value_d").is_some() {
                    "\""
                } else {
                    "'"
                };
                format!(
                    "{}{}{}{}",
                    standardized_prefix,
                    quote,
                    path.display(),
                    quote
                )
            })
            .to_string()
        } else {
            // If no match is found, append the new path line at the end.
            let mut new_content = config_content.clone();
            new_content.push('\n');
            new_content.push_str(&format!("path: \"{}\"", path.display()));
            new_content
        };

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(file_path)?;
        file.write_all(updated_content.as_bytes())?;

        Ok(())
    }

    pub fn find() -> Result<PathBuf, TapError> {
        let mut paths = vec![];

        if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
            let xdg_config_home = PathBuf::from(xdg_config_home);
            paths.push(xdg_config_home.join("tap").join("tap.yml"));
            paths.push(xdg_config_home.join("tap.yml"));
        }

        if let Ok(home_dir) = env::var("HOME") {
            let home_dir = PathBuf::from(home_dir);
            paths.push(home_dir.join(".config").join("tap").join("tap.yml"));
            paths.push(home_dir.join(".tap.yml"));
        }

        for path in paths {
            if path.exists() {
                return Ok(path);
            }
        }

        bail!("Config file not found!")
    }

    pub fn deserialize() -> Result<Self, TapError> {
        let config_path = FileConfig::find()?;
        let mut file = fs::File::open(config_path)?;
        let mut contents = String::new();
        io::Read::read_to_string(&mut file, &mut contents)?;
        let file_config = serde_yaml::from_str(&contents)?;

        Ok(file_config)
    }

    pub fn load_keybindings_only() -> Result<HashMap<String, Vec<String>>, TapError> {
        let config_path = FileConfig::find()?;
        let mut file = fs::File::open(config_path)?;
        let mut contents = String::new();
        io::Read::read_to_string(&mut file, &mut contents)?;
        let res: KeybindingOnly = serde_yaml::from_str(&contents)?;

        res.keybinding
            .ok_or(anyhow::anyhow!("Failed to load keybindings from config"))
    }

    pub fn expanded_path(&self) -> Option<PathBuf> {
        self.path.as_ref().map(|p| {
            let path_str = p.to_string_lossy();
            if path_str.starts_with("~/") {
                if let Ok(home) = env::var("HOME") {
                    return Path::new(&home).join(&path_str[2..]);
                }
            }
            p.clone()
        })
    }
}
