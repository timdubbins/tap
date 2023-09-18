use std::collections::VecDeque;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::Duration;

use anyhow::bail;
use cursive::event::{Event, EventResult, EventTrigger, Key, MouseButton, MouseEvent};

use crate::args::{Args, Opts};
use crate::fuzzy::*;
use crate::player::{Player, PlayerOpts};
use crate::serde::*;
use crate::utils::{CycleIterator, IntoInner};
use crate::views::{FuzzyView, PlayerView};

struct UserData {
    opts: PlayerOpts,
    paths: Vec<PathBuf>,
    queue: VecDeque<(PathBuf, usize)>,
}

impl UserData {
    fn new(path: &PathBuf, items: &Vec<FuzzyItem>) -> Result<Self, anyhow::Error> {
        let paths = leaf_paths(&items);
        let queue: VecDeque<(PathBuf, usize)> = match Player::randomized(&paths) {
            Some(first) => VecDeque::from([first]),
            None => bail!("could not find audio files in '{}'", path.display()),
        };

        let data = Self {
            opts: PlayerOpts::default(),
            paths,
            queue,
        };

        Ok(data)
    }
}

impl IntoInner for UserData {
    type T = ((u8, u8, bool), Vec<PathBuf>, VecDeque<(PathBuf, usize)>);

    fn into_inner(self) -> Self::T {
        (self.opts.into_inner(), self.paths, self.queue)
    }
}

impl Into<UserData> for ((u8, u8, bool), Vec<PathBuf>, VecDeque<(PathBuf, usize)>) {
    fn into(self) -> UserData {
        UserData {
            opts: self.0.into(),
            paths: self.1,
            queue: self.2,
        }
    }
}

pub struct App {}

impl App {
    pub fn run() -> Result<(), anyhow::Error> {
        let (path, opts) = Args::parse_args()?;

        match opts {
            Opts::Automate => return run_automated(&path),
            Opts::Set => return process_cache(&path, "setting default"),
            Opts::Print => return print_cached_path(),
            _ => (),
        }

        // The items to fuzzy search on, if any.
        let items = get_items(&path, opts)?;

        // The cursive root.
        let mut siv = cursive::ncurses();

        // Set style and background color.
        siv.load_toml(include_str!("assets/style.toml"))
            .expect("style.toml should be located in assets directory");

        // Set the refresh rate.
        siv.set_fps(15);

        if items.is_empty() {
            // There are no items to search on so run a standalone player.
            let (player, size) = Player::new(&path)?;
            PlayerView::load((player, size), &mut siv);
            siv.run();
            return Ok(());
        }

        // Load the initial fuzzy search.
        FuzzyView::load(items.to_owned(), &mut siv);

        // Set the initial user data.
        let user_data = UserData::new(&path, &items)?;
        siv.set_user_data(user_data.into_inner());

        // Set the callback for the previous selection.
        siv.set_on_pre_event_inner('-', |_| {
            Some(EventResult::with_cb(|siv| {
                PlayerView::previous(false, siv);
            }))
        });

        // Set callback for a random selection.
        siv.set_on_pre_event_inner('=', |_| {
            Some(EventResult::with_cb(|siv| {
                PlayerView::random(false, siv);
            }))
        });

        // Set the callbacks for the fuzzy-finder.
        siv.set_on_pre_event_inner(trigger(), move |event: &Event| {
            let c = event.char().unwrap_or('0');
            let items = items.to_owned();

            if matches!(c, 'A'..='Z') {
                let items = key_items(c, items);
                return Some(EventResult::with_cb(move |siv| {
                    FuzzyView::with(items.to_owned(), c, siv)
                }));
            }

            let items = match c {
                'a' => non_leaf_items(items),
                's' => leaf_items(items),
                _ => match event.f_num() {
                    Some(depth) => depth_items(depth, items),
                    None => items,
                },
            };

            Some(EventResult::with_cb(move |siv| {
                FuzzyView::load(items.to_owned(), siv)
            }))
        });

        siv.run();
        Ok(())
    }
}

// Runs an automated player in the command line without the TUI.
fn run_automated(path: &PathBuf) -> Result<(), anyhow::Error> {
    let (mut player, _) = Player::new(path)?;
    let (mut line, mut length) = player.stdout();

    print!("{}", line);
    stdout().flush()?;

    loop {
        match player.poll_sink() {
            0 => return Ok(()),
            1 => {
                // Print the number of spaces required to clear the previous line.
                print!("\r{: <1$}", "", length);
                (line, length) = player.stdout();
                print!("\r{}", line);
                stdout().flush()?;
            }
            _ => sleep(Duration::from_millis(60)),
        }
    }
}

fn process_cache(path: &PathBuf, action: &'static str) -> Result<(), anyhow::Error> {
    match process(update_cache, path, action) {
        Ok(_) => {
            println!("\r[tap]: {}...", action);
            println!("[tap]: done!");
            return Ok(());
        }
        Err(e) => bail!(e),
    }
}

fn print_cached_path() -> Result<(), anyhow::Error> {
    let cached_path = get_cached::<PathBuf>("path")?;
    println!("[tap]: default set to '{}'", cached_path.display());

    Ok(())
}

fn get_items(path: &PathBuf, opts: Opts) -> Result<Vec<FuzzyItem>, anyhow::Error> {
    match opts == Opts::Default || uses_default(path) {
        true => match needs_update(path)? {
            true => process(update_cache, path, "updating"),
            false => get_cached::<Vec<FuzzyItem>>("items"),
        },
        false => process(create_items, path, "loading"),
    }
}

fn process(
    items: fn(&PathBuf) -> Result<Vec<FuzzyItem>, anyhow::Error>,
    path: &PathBuf,
    action: &'static str,
) -> Result<Vec<FuzzyItem>, anyhow::Error> {
    let (tx, rx) = mpsc::channel();

    let stdout_handle = thread::spawn(move || {
        let ellipses = vec!["   ", ".  ", ".. ", "..."];
        let mut spinner = CycleIterator::new(ellipses);

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
    });

    let items = items(path);

    tx.send(true)?;
    stdout_handle.join().unwrap();

    items
}

// Trigger for the fuzzy-finder callbacks.
fn trigger() -> EventTrigger {
    EventTrigger::from_fn(|event| {
        matches!(
            event,
            Event::Key(Key::Tab)
                | Event::Char('A'..='Z')
                | Event::CtrlChar('a')
                | Event::CtrlChar('s')
                | Event::Key(Key::F1)
                | Event::Key(Key::F2)
                | Event::Key(Key::F3)
                | Event::Key(Key::F4)
                | Event::Mouse {
                    event: MouseEvent::Press(MouseButton::Middle),
                    ..
                }
        )
    })
}
