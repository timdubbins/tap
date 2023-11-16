use std::collections::VecDeque;
use std::ops::Range;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::bail;
use rand::{thread_rng, Rng};

pub trait IntoInner {
    type T;
    fn into_inner(self) -> Self::T;
}

pub type UserData = (
    (u8, u8, bool, bool),
    Vec<PathBuf>,
    VecDeque<(PathBuf, usize)>,
);

// Maps the array to a single value, i.e. `[0, 1, 2]` -> `12`.
pub fn concatenate(arr: &Vec<usize>) -> usize {
    arr.iter().fold(0, |acc, x| acc * 10 + x)
}

// Generates a random unsigned int in the given range.
pub fn random(range: Range<usize>) -> usize {
    thread_rng().gen_range(range)
}

// Bounds a value by a minimum and maximum value.
pub fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

// Gets the last modification time listed in the metadata for the path.
pub fn last_modified(path: &PathBuf) -> Result<SystemTime, anyhow::Error> {
    match std::fs::metadata(&path) {
        Ok(data) => match data.modified() {
            Ok(time) => Ok(time),
            Err(e) => bail!(e),
        },
        Err(e) => bail!(e),
    }
}

// Attempts to open the path with the default file manager.
// Requires 'xdg-open' on linux systems. Uses 'open' on macos.
pub fn open_file_manager(path: PathBuf) -> Result<(), anyhow::Error> {
    let p = match std::fs::metadata(&path) {
        Ok(meta) => match meta.is_dir() {
            true => path,
            // false => path.parent().unwrap().to_path_buf(),
            false => match path.parent() {
                Some(parent) => parent.to_path_buf(),
                None => bail!("No parent"),
            },
        },
        Err(err) => bail!(err),
    };

    let s = p
        .as_os_str()
        .to_str()
        .expect("should be a valid UTF-8 path");

    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("open").arg(s).status();
        match status {
            Ok(_) => Ok(()),
            Err(err) => bail!(err),
        }
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("xdg-open").arg(s).status();
        match status {
            Ok(_) => Ok(()),
            Err(err) => bail!(err),
        }
    }
}

#[cfg(test)]
// Find the test assets.
pub fn find_assets_dir() -> PathBuf {
    // Tests exe is in target/debug/deps, test assets are in tests/assets
    let root = std::env::current_exe()
        .expect("tests executable")
        .parent()
        .expect("tests executable directory")
        .parent()
        .expect("tap executable directory")
        .parent()
        .expect("target directory")
        .parent()
        .expect("project root")
        .to_path_buf();

    root.join("tests").join("assets")
}

#[cfg(test)]
// Create the working directory and test data.
pub fn create_working_dir(
    dirs: &[&'static str],
    audio_data: &[(&'static str, &'static str)],
    dummy_data: &[&'static str],
) -> Result<tempfile::TempDir, std::io::Error> {
    let temp_dir = tempfile::Builder::new()
        .prefix("tap-tests")
        .tempdir()
        .expect("failed to create temporary directory");

    let assets_dir = find_assets_dir();

    for path in dirs {
        let path = temp_dir.path().join(path);
        std::fs::create_dir_all(path).expect("failed to create subdirectories")
    }

    for (temp_path, asset_path) in audio_data {
        let src = assets_dir.join(asset_path);
        let dest = temp_dir.path().join(temp_path);
        std::fs::copy(src, dest).expect("failed to copy audio data");
    }

    for path in dummy_data {
        let path = temp_dir.path().join(path);
        std::fs::File::create(path).expect("failed to create dummy data");
    }

    Ok(temp_dir)
}
