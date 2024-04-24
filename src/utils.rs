use std::{
    io::{stdout, Write},
    ops::Range,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime},
};

use anyhow::bail;
use rand::{thread_rng, Rng};

pub trait IntoInner {
    type T;
    fn into_inner(self) -> Self::T;
}

pub type InnerType<U> = <U as IntoInner>::T;

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

pub fn display_with_spinner<F, T>(
    action: F,
    path: &PathBuf,
    msg: &'static str,
) -> Result<T, anyhow::Error>
where
    F: FnOnce(&PathBuf) -> Result<T, anyhow::Error> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let start_time = Instant::now();

    let stdout_handle = thread::spawn(move || {
        let ellipses = vec!["   ", ".  ", ".. ", "..."];
        let mut spinner = ellipses.iter().cycle();
        let mut is_showing = false;

        loop {
            match rx.try_recv() {
                Ok(should_exit) => {
                    if should_exit {
                        print!("\r{: <1$}\r", "", 20);
                        stdout().flush().unwrap_or_default();
                        break;
                    }
                }
                Err(_) => {
                    if is_showing {
                        print!("\r[tap]: {}{} ", msg, spinner.next().unwrap());
                        stdout().flush().unwrap();
                    }
                    thread::sleep(Duration::from_millis(300));
                }
            }

            if !is_showing && start_time.elapsed() > Duration::from_millis(300) {
                is_showing = true;
            }
        }
    });

    let result = action(path);
    tx.send(true)?;
    stdout_handle.join().unwrap();

    result
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
