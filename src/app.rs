use std::io::{stdout, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::Duration;

use anyhow::bail;
use cursive::event::{Event, EventResult, EventTrigger, Key, MouseButton, MouseEvent};
use cursive::CursiveRunnable;

use crate::args::{self, Opts};
use crate::data::UserData;
use crate::fuzzy::{self, FuzzyItem, FuzzyView};
use crate::player::{PlayerBuilder, PlayerView};
use crate::serde;
use crate::theme;
use crate::utils::IntoInner;

// Run the app.
pub fn run() -> Result<(), anyhow::Error> {
    let (path, opts) = args::parse()?;

    match opts {
        Opts::Automate => return run_automated(path),
        Opts::Set => return process_cache(path, "setting default"),
        Opts::Print => return print_cached_path(),
        _ => (),
    }

    // The items to fuzzy search on.
    let items = get_items(&path, opts)?;

    // The cursive root.
    let mut siv = cursive::ncurses();

    siv.set_theme(theme::custom());
    siv.set_fps(15);

    if items.len() < 2 {
        return run_standalone(items, path, siv);
    }

    // Load the initial fuzzy search.
    FuzzyView::load(items.to_owned(), None, &mut siv);

    // Set the initial user data.
    let user_data = UserData::new(&path, &items)?;
    siv.set_user_data(user_data.into_inner());

    siv.set_on_pre_event_inner('-', previous_album);
    siv.set_on_pre_event_inner('=', random_album);

    // Set the callbacks for the fuzzy-finder.
    siv.set_on_pre_event_inner(trigger(), move |event: &Event| {
        let key = event.char();
        let (items, key) = match key {
            Some('A'..='Z') => (fuzzy::key_items(key, &items), key),
            Some('a') => (fuzzy::non_leaf_items(&items), None),
            Some('s') => (fuzzy::audio_items(&items), None),
            _ => match event.f_num() {
                Some(depth) => (fuzzy::depth_items(depth, &items), None),
                None => (items.to_owned(), None),
            },
        };
        Some(EventResult::with_cb(move |siv| {
            FuzzyView::load(items.to_owned(), key, siv)
        }))
    });

    handle_runner(siv)
}

fn handle_runner(mut siv: CursiveRunnable) -> Result<(), anyhow::Error> {
    // Exit the process in test builds.
    #[cfg(feature = "run_tests")]
    {
        match siv.user_data::<crate::utils::UserData>() {
            // Output user data as stderr, if available.
            Some(user_data) => bail!("{:?}", user_data),
            None => Ok(()),
        }
    }

    // Run the Cursive event loop in non-test builds.
    #[cfg(not(feature = "run_tests"))]
    {
        siv.run();
        Ok(())
    }
}

// Runs a standalone player without the fuzzy finder.
fn run_standalone(
    items: Vec<FuzzyItem>,
    path: PathBuf,
    mut siv: CursiveRunnable,
) -> Result<(), anyhow::Error> {
    let path = match items.first() {
        Some(item) => item.path.to_owned(),
        None => path,
    };
    let player = PlayerBuilder::new(path)?;
    PlayerView::load(player, &mut siv);

    handle_runner(siv)
}

// Run an automated player in the command line without the TUI.
fn run_automated(path: PathBuf) -> Result<(), anyhow::Error> {
    let (mut player, _, _) = PlayerBuilder::new(path)?;
    let (mut line, mut length) = player.stdout();

    print!("{}", line);
    stdout().flush()?;

    loop {
        match player.poll() {
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

fn process_cache(path: PathBuf, action: &'static str) -> Result<(), anyhow::Error> {
    match process(serde::update_cache, &path, action) {
        Ok(_) => {
            println!("\r[tap]: {}...", action);
            println!("[tap]: done!");
            return Ok(());
        }
        Err(e) => bail!(e),
    }
}

fn print_cached_path() -> Result<(), anyhow::Error> {
    let cached_path = serde::cached_path()?;
    println!("[tap]: default set to '{}'", cached_path.display());

    Ok(())
}

fn get_items(path: &PathBuf, opts: Opts) -> Result<Vec<FuzzyItem>, anyhow::Error> {
    let items = match opts == Opts::Default || serde::uses_default(path) {
        true => match serde::needs_update(path)? {
            true => process(serde::update_cache, path, "updating"),
            false => match serde::cached_items() {
                Ok(items) => Ok(items),
                // Try an update before bailing.
                Err(_) => process(serde::update_cache, path, "updating"),
            },
        },
        false => process(fuzzy::create_items, path, "loading"),
    }?;

    if args::audio_only() {
        Ok(fuzzy::audio_items(&items))
    } else {
        Ok(items)
    }
}

fn process(
    action: fn(&PathBuf) -> Result<Vec<FuzzyItem>, anyhow::Error>,
    path: &PathBuf,
    msg: &'static str,
) -> Result<Vec<FuzzyItem>, anyhow::Error> {
    let (tx, rx) = mpsc::channel();

    let stdout_handle = thread::spawn(move || {
        let ellipses = vec!["   ", ".  ", ".. ", "..."];
        let mut spinner = ellipses.iter().cycle();

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
                    print!("\r[tap]: {}{} ", msg, spinner.next().unwrap());
                    stdout().flush().unwrap();
                    sleep(Duration::from_millis(300));
                }
            }
        }
    });

    let items = action(path);

    tx.send(true)?;
    stdout_handle.join().unwrap();

    items
}

// Callback to select the previous album.
fn previous_album(_: &Event) -> Option<EventResult> {
    Some(EventResult::with_cb(|siv| {
        if let Ok(player) = PlayerBuilder::PreviousAlbum.from(None, siv) {
            PlayerView::load(player, siv);
        }
    }))
}

// Callback to select a random album.
fn random_album(_: &Event) -> Option<EventResult> {
    Some(EventResult::with_cb(|siv| {
        if let Ok(player) = PlayerBuilder::RandomAlbum.from(None, siv) {
            PlayerView::load(player, siv);
        }
    }))
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
