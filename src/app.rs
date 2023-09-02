use std::collections::VecDeque;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use anyhow::bail;
use cursive::event::{Event, EventResult, EventTrigger, Key, MouseButton, MouseEvent};

use crate::args::Args;
use crate::fuzzy::*;
use crate::player::Player;
use crate::views::{FuzzyView, PlayerView};

#[derive(Clone)]
pub struct App {}

impl App {
    pub fn run() -> Result<(), anyhow::Error> {
        // The initial path to play or search on.
        let path = Args::parse_path()?;

        if Args::is_automated() {
            return App::run_automated(&path);
        }

        // The items to fuzzy search on, if any.
        let items = get_items(&path);

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
        }

        // Load the initial fuzzy search.
        FuzzyView::load(items.to_owned(), &mut siv);

        // The initial user data.
        let paths = leaf_paths(&items);
        let queue: VecDeque<(PathBuf, usize)> = match Player::randomized(&paths) {
            Some(first) => VecDeque::from([first]),
            None => bail!("could not find a randomized track"),
        };

        // Set the initial user data.
        siv.set_user_data((paths, queue));

        // Set the callback for the previous selection.
        siv.set_on_pre_event_inner('-', |_| {
            Some(EventResult::with_cb(|siv| {
                PlayerView::previous(None, siv);
            }))
        });

        // Set callback for a random selection.
        siv.set_on_pre_event_inner('=', |_| {
            Some(EventResult::with_cb(|siv| {
                PlayerView::random(None, siv);
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
