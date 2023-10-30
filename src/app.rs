use std::io::{stdout, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::Duration;

use anyhow::bail;
use cursive::event::{Event, EventResult, EventTrigger, Key, MouseButton, MouseEvent};

use crate::args::{parse_args, Opts};
use crate::data::UserData;
use crate::player::{PlayerBuilder, PlayerView};
use crate::serialization::*;
use crate::utils::IntoInner;
use crate::{fuzzy::*, theme};

// Runs the app.
pub fn run() -> Result<(), anyhow::Error> {
    let (path, opts) = parse_args()?;

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
        let player = PlayerBuilder::new(path)?;
        PlayerView::load(player, &mut siv);

        return handle_runner(siv);
    }

    // Load the initial fuzzy search.
    FuzzyView::load(items.to_owned(), &mut siv);

    // Set the initial user data.
    let user_data = UserData::new(&path, &items)?;
    siv.set_user_data(user_data.into_inner());

    // Set the callback for the previous selection.
    siv.set_on_pre_event_inner('-', |_| {
        Some(EventResult::with_cb(|siv| {
            if let Ok(player) = PlayerBuilder::PreviousAlbum.from(None, siv) {
                PlayerView::load(player, siv);
            }
        }))
    });

    // Set callback for a random selection.
    siv.set_on_pre_event_inner('=', |_| {
        Some(EventResult::with_cb(|siv| {
            if let Ok(player) = PlayerBuilder::RandomAlbum.from(None, siv) {
                PlayerView::load(player, siv);
            }
        }))
    });

    // Set the callbacks for the fuzzy-finder.
    siv.set_on_pre_event_inner(trigger(), move |event: &Event| {
        let c = event.char().unwrap_or('0');

        if matches!(c, 'A'..='Z') {
            let items = key_items(c, &items);
            return Some(EventResult::with_cb(move |siv| {
                FuzzyView::with(items.to_owned(), c, siv)
            }));
        }

        let items = match c {
            'a' => non_leaf_items(&items),
            's' => audio_items(&items),
            _ => match event.f_num() {
                Some(depth) => depth_items(depth, &items),
                None => items.to_owned(),
            },
        };

        Some(EventResult::with_cb(move |siv| {
            FuzzyView::load(items.to_owned(), siv)
        }))
    });

    handle_runner(siv)
}

fn handle_runner(mut siv: cursive::CursiveRunnable) -> Result<(), anyhow::Error> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "run_tests")] {
            // Exit the process in test builds.
            match siv.user_data::<crate::utils::UserData>() {
                // Output user data as stderr, if available.
                Some(user_data) => bail!("{:?}", user_data),
                None => Ok(()),
            }
        } else {
            // Run the Cursive event loop in production builds.
            siv.run();
            Ok(())
        }
    }
}

// Runs an automated player in the command line without the TUI.
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
    match process(update_cache, &path, action) {
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
            false => match get_cached::<Vec<FuzzyItem>>("items") {
                Ok(items) => Ok(items),
                // Try an update before bailing.
                Err(_) => process(update_cache, path, "updating"),
            },
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
