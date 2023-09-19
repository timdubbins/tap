use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::{ops::Range, time::SystemTime};

use anyhow::bail;
use rand::{thread_rng, Rng};

pub trait IntoInner {
    type T;
    fn into_inner(self) -> Self::T;
}

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

pub fn has_child(path: &PathBuf) -> bool {
    let iter = match path.read_dir() {
        Ok(r) => r,
        Err(_) => return false,
    };
    for entry in iter {
        if let Ok(entry) = entry {
            if entry.path().is_dir() {
                return true;
            }
        }
    }
    false
}

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

    pub fn toggle(&mut self) {
        self.ignores_timer ^= true;
    }

    pub fn set(&mut self) {
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

// Returns true if the path has at least two children.
pub fn has_child_dirs(path: &PathBuf) -> bool {
    let mut has_child_dir = false;

    let iter = match path.read_dir() {
        Ok(r) => r,
        Err(_) => return false,
    };

    for entry in iter {
        if let Ok(entry) = entry {
            if entry.path().is_dir() {
                if has_child_dir {
                    // The second child is found.
                    return true;
                } else {
                    // The first child is found.
                    has_child_dir = true;
                }
            }
        }
    }
    false
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
