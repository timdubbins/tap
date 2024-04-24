use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    time::SystemTime,
};

use anyhow::bail;
use bincode::{config, Decode};

use crate::fuzzy::{self, FuzzyItem};
use crate::utils;

pub fn cached_path() -> Result<PathBuf, anyhow::Error> {
    // ~/.cache/tap/path
    get_cached::<PathBuf>("path")
}

pub fn cached_items() -> Result<Vec<FuzzyItem>, anyhow::Error> {
    // ~/.cache/tap/items
    get_cached::<Vec<FuzzyItem>>("items")
}

fn cached_last_modified() -> Result<SystemTime, anyhow::Error> {
    // ~/.cache/tap/last_modified
    get_cached::<SystemTime>("last_modified")
}

pub fn needs_update(path: &PathBuf) -> Result<bool, anyhow::Error> {
    let res = utils::last_modified(path)?.eq(&cached_last_modified()?);
    Ok(!res)
}

pub fn uses_default(path: &PathBuf) -> bool {
    let cached_path = cached_path().unwrap_or_default();
    cached_path.eq(path)
}

fn get_cached<T: Decode>(file_name: &str) -> Result<T, anyhow::Error> {
    let file_path = cache_dir()?.join(file_name);

    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            bail!("\r[tap error]: use '--set-default' to set a default directory")
        }
    };
    let mut encoded = Vec::new();
    file.read_to_end(&mut encoded)?;

    let config = config::standard();
    let (ret, _): (T, _) = bincode::decode_from_slice(&encoded[..], config)?;

    Ok(ret)
}

fn cache_dir() -> Result<PathBuf, anyhow::Error> {
    let home_dir = match std::env::var("HOME") {
        Ok(dir) => PathBuf::from(dir),
        Err(e) => bail!(e),
    };

    let cache_dir = home_dir.join(".cache").join("tap");
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

pub fn update_cache(path: &PathBuf) -> Result<Vec<FuzzyItem>, anyhow::Error> {
    let last_modified = utils::last_modified(path)?;
    let items = fuzzy::create_items(path)?;

    let config = config::standard();
    let cache_dir = cache_dir()?;

    let encoded_path = bincode::encode_to_vec(path, config)?;
    let encoded_modified = bincode::encode_to_vec(last_modified, config)?;
    let encoded_items = bincode::encode_to_vec(items.clone(), config)?;

    let mut path = File::create(cache_dir.join("path"))?;
    path.write_all(&encoded_path)?;

    let mut last_modified = File::create(cache_dir.join("last_modified"))?;
    last_modified.write_all(&encoded_modified)?;

    let mut items_file = File::create(cache_dir.join("items"))?;
    items_file.write_all(&encoded_items)?;

    Ok(items)
}

pub fn get_cached_items(path: &PathBuf) -> Result<Vec<FuzzyItem>, anyhow::Error> {
    Ok(match needs_update(path)? {
        true => utils::display_with_spinner(update_cache, path, "updating")?,
        false => match cached_items() {
            Ok(items) => items,
            // Try an update before bailing.
            Err(_) => utils::display_with_spinner(update_cache, path, "updating")?,
        },
    })
}

pub fn set_default_path(path: PathBuf) -> Result<(), anyhow::Error> {
    let msg = "setting default";
    match utils::display_with_spinner(update_cache, &path, msg) {
        Ok(_) => {
            println!("\r[tap]: {}...", msg);
            println!("[tap]: done!");
            return Ok(());
        }
        Err(e) => bail!(e),
    }
}

pub fn print_default_path() -> Result<(), anyhow::Error> {
    let cached_path = cached_path()?;
    println!("[tap]: default set to '{}'", cached_path.display());

    Ok(())
}
