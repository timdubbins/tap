use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::bail;
use bincode::{config, Decode};

use crate::fuzzy::{create_items, FuzzyItem};
use crate::utils::last_modified;

// ~/.cache/tap/path
// ~/.cache/tap/last_modified
// ~/.cache/tap/items

// benchmark:
// test directory size: 200GB, cache size: 350KB
// load test directory: 1.8s, load cache: 0.3s

pub fn get_cached<T: Decode>(file_name: &str) -> Result<T, anyhow::Error> {
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

pub fn needs_update(path: &PathBuf) -> Result<bool, anyhow::Error> {
    let cached_modified = get_cached::<SystemTime>("last_modified")?;
    Ok(!last_modified(path)?.eq(&cached_modified))
}

pub fn uses_default(path: &PathBuf) -> bool {
    let cached_path = get_cached::<PathBuf>("path").unwrap_or_default();
    cached_path.eq(path)
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
    let last_modified = last_modified(path)?;
    let items = create_items(path)?;

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
