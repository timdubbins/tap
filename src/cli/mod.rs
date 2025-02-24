pub mod args;
pub mod logger;
pub mod player;

use std::path::PathBuf;

use anyhow::anyhow;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{config::FileConfig, finder::Library, TapError};

pub use self::{args::Args, logger::Logger};

const REPO_URL: &str = "https://api.github.com/repos/timdubbins/tap/releases/latest";

pub struct Cli {}

impl Cli {
    pub fn set_cache(search_root: &PathBuf) -> Result<(), TapError> {
        FileConfig::find()?;
        let logger = Logger::start("setting default");
        let library = Library::new(search_root);
        library.serialize()?;
        FileConfig::update_path(search_root)?;
        logger.stop();
        Ok(())
    }

    pub fn print_cache() -> Result<(), TapError> {
        let file_config = FileConfig::deserialize()?;

        let path = file_config
            .path
            .ok_or_else(|| anyhow!("Path not set in config file!"))?;

        println!("[tap]: default path: {:?}", path);
        Ok(())
    }

    pub fn check_version() -> Result<(), TapError> {
        let prefix = "[tap]:";

        match Self::fetch_latest_version() {
            Ok(latest_version) if crate::config::VERSION == latest_version => {
                println!(
                    "{} You're using the latest version: {}",
                    prefix,
                    crate::config::VERSION
                );
            }
            Ok(latest_version) => {
                println!(
                    "{} You're using version: {}. A new version is available: {}",
                    prefix,
                    crate::config::VERSION,
                    latest_version
                );
            }
            Err(_) => {
                println!(
                    "{} You're using version: {}",
                    prefix,
                    crate::config::VERSION
                );
            }
        }

        Ok(())
    }

    fn fetch_latest_version() -> Result<String, TapError> {
        #[derive(Deserialize)]
        struct GitHubRelease {
            tag_name: String,
        }

        let client = Client::builder().user_agent("tap").build()?;
        let response = client.get(REPO_URL).send()?.json::<GitHubRelease>()?;

        let version = response
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&response.tag_name)
            .to_string();

        Ok(version)
    }
}
