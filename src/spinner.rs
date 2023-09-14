use std::{
    io::{stdout, Write},
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

use crate::args::Args;

struct CircularIterator<T: Clone> {
    items: Vec<T>,
    current_index: usize,
}

impl<T: Clone> CircularIterator<T> {
    fn new(items: Vec<T>) -> Self {
        Self {
            items,
            current_index: 0,
        }
    }
}

impl<T: Clone> Iterator for CircularIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.items.is_empty() {
            return None;
        }

        let next_item = self.items[self.current_index].clone();
        self.current_index = (self.current_index + 1) % self.items.len();

        Some(next_item)
    }
}

pub fn spinner_stdout(rx: mpsc::Receiver<bool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let ellipses = vec!["   ", ".  ", ".. ", "..."];
        let mut spinner = CircularIterator::new(ellipses);

        let action = match Args::set_current() {
            true => "saving",
            false => "loading",
        };

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
                    print!("\r[tap]: {}{} ", action, spinner.next().unwrap());
                    stdout().flush().unwrap();
                    sleep(Duration::from_millis(300));
                }
            }
        }
    })
}
