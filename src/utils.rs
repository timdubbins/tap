use std::{
    collections::VecDeque,
    ops::Range,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime},
};

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

// An iterator that cycles back to the first element.
pub struct CycleIterator<T> {
    items: Vec<T>,
    index: usize,
}

impl<T> CycleIterator<T> {
    pub fn new(items: Vec<T>) -> Self {
        CycleIterator { items, index: 0 }
    }
}

impl<T: Clone> Iterator for CycleIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.items.is_empty() {
            return None;
        }

        let item = self.items[self.index].clone();
        self.index = (self.index + 1) % self.items.len();
        Some(item)
    }
}

// A boolean that automatically switches to false after a set `duration`.
//
// Functions like a regular boolean if `ignores_timer` is true.
pub struct TimerBool {
    last_set: Arc<Mutex<Instant>>,
    value: Arc<AtomicBool>,
    duration: Duration,
    ignores_timer: bool,
}

impl TimerBool {
    pub fn new(v: bool, duration: Duration) -> Self {
        TimerBool {
            value: Arc::new(AtomicBool::new(v)),
            last_set: Arc::new(Mutex::new(Instant::now())),
            ignores_timer: false,
            duration,
        }
    }

    pub fn set_false(&self) {
        self.value.store(false, Ordering::Relaxed);
    }

    pub fn is_true(&self) -> bool {
        self.ignores_timer || self.value.load(Ordering::Relaxed)
    }

    pub fn toggle(&mut self) -> bool {
        if self.value.load(Ordering::Relaxed) {
            self.value.store(false, Ordering::Relaxed);
            false
        } else {
            self.ignores_timer ^= true;
            self.ignores_timer
        }
    }

    pub fn set(&mut self) {
        if self.ignores_timer {
            return;
        }

        let last_set = self.last_set.lock().unwrap().clone();
        let now = Instant::now();
        let elapsed = now.duration_since(last_set);

        if elapsed > self.duration || !self.value.load(Ordering::Relaxed) {
            self.value.store(true, Ordering::Relaxed);
            *self.last_set.lock().unwrap() = now;

            // Spawn a new thread to reset the boolean after the specified timeout
            let value_clone = Arc::clone(&self.value);
            let last_set_clone = Arc::clone(&self.last_set);
            let duration_clone = self.duration.clone();

            thread::spawn(move || loop {
                let last_set = last_set_clone.lock().unwrap().clone();
                let now = Instant::now();
                let elapsed = now.duration_since(last_set);

                if elapsed > duration_clone {
                    value_clone.store(false, Ordering::Relaxed);
                    break;
                } else {
                    thread::sleep(Duration::from_millis(50));
                }
            });
        } else {
            *self.last_set.lock().unwrap() = now;
        }
    }
}

// Maps the array to a single value, i.e. `[0, 1, 2]` -> `12`.
pub fn concatenate(arr: &Vec<usize>) -> usize {
    arr.iter().fold(0, |acc, x| acc * 10 + x)
}

// Generates a random unsigned int in the given range.
pub fn random(range: Range<usize>) -> usize {
    thread_rng().gen_range(range)
}

pub fn last_modified(path: &PathBuf) -> Result<SystemTime, anyhow::Error> {
    match std::fs::metadata(&path) {
        Ok(data) => match data.modified() {
            Ok(time) => Ok(time),
            Err(e) => bail!(e),
        },
        Err(e) => bail!(e),
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
// Create the working directory and the test files.
pub fn create_working_dir(
    dirs: &[&'static str],
    audio_data: &[&'static str],
    dummy_data: &[&'static str],
) -> Result<tempfile::TempDir, std::io::Error> {
    let temp_dir = tempfile::Builder::new().prefix("tap-tests").tempdir()?;
    let root = temp_dir.path();
    let audio = find_assets_dir().join("test_audio_1.mp3");

    for path in dirs {
        std::fs::create_dir_all(root.join(path))?;
    }

    for path in audio_data {
        std::fs::copy(&audio, root.join(path)).expect("failed to copy file");
    }

    for path in dummy_data {
        std::fs::File::create(root.join(path))?;
    }

    Ok(temp_dir)
}
