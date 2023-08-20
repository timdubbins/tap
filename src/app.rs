use std::collections::VecDeque;
use std::path::PathBuf;

use cursive::event::{Event, EventResult, EventTrigger, Key, MouseButton, MouseEvent};
use cursive::Cursive;

use crate::args::Args;
use crate::fuzzy::*;
use crate::player::Player;
use crate::utils::*;
use crate::views::{FuzzyView, PlayerView};

#[derive(Clone)]
pub struct App {}

impl App {
    pub fn run() -> Result<(), anyhow::Error> {
        // The initial path to play or search on.
        let path = Args::parse_path()?;
        // The items to fuzzy search on, if any.
        let items = get_items(&path);
        // The cursive root.
        let mut siv = cursive::default();

        // Set style and background color.
        siv.load_toml(include_str!("assets/style.toml"))
            .expect("style.toml should be located in assets directory");

        // Set the refresh rate.
        siv.set_fps(15);

        // There are no items to search on so load `path` into the player and run.
        if items.is_empty() {
            let (player, size) = Player::new(path.to_owned())?;
            PlayerView::load((player, size), &mut siv);
            siv.run();
            return Ok(());
        }

        // Load the initial fuzzy search.
        FuzzyView::load(items.to_owned(), &mut siv);

        // Register the inner callbacks for fuzzy searching and selection.
        siv.set_on_pre_event_inner(trigger(), move |event: &Event| {
            let c = event.char().unwrap_or('0');

            let items = match c {
                'A'..='Z' => key_items(c, items.to_owned()),
                'a' => leaf_items(items.to_owned()),
                's' => non_leaf_items(items.to_owned()),
                _ => match event.f_num() {
                    Some(depth) => depth_items(depth, items.to_owned()),
                    None => items.to_owned(),
                },
            };

            Some(EventResult::with_cb(move |siv| {
                if c.eq(&'=') {
                    random_selection(&items, siv)
                } else if c.eq(&'-') {
                    previous_selection(siv)
                } else {
                    FuzzyView::load(items.to_owned(), siv)
                }
            }))
        });

        siv.run();
        Ok(())
    }
}

// Trigger for inner callbacks.
fn trigger() -> EventTrigger {
    EventTrigger::from_fn(|event| {
        matches!(
            event,
            Event::Key(Key::Tab)
                | Event::Char('=')
                | Event::Char('-')
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

// Selects and loads a leaf directory at random. Invalid selections are ignored.
fn random_selection(items: &Vec<FuzzyItem>, siv: &mut Cursive) {
    let mut count = 0;
    let curr_path = curr_path(siv);

    // Loop until we find a valid selection or we give up.
    while count < 10 {
        let random_path = random_path(items);
        if Some(random_path.clone()).eq(&curr_path) || has_child_dirs(&random_path) {
            count += 1
        } else if let Ok((player, size)) = Player::new(random_path) {
            PlayerView::load((player, size), siv);
            break;
        } else {
            count += 1;
        }
    }
}

// Returns a randomly selected path from `items`.
fn random_path(items: &Vec<FuzzyItem>) -> PathBuf {
    let items = items;
    let target = random(0..items.len() - 1);
    items[target].path.to_owned()
}

// Selects and loads the previous player.
fn previous_selection(siv: &mut Cursive) {
    if let Some(path) = prev_path(siv) {
        let player = Player::new(path).expect("should load a previous player");
        PlayerView::load(player, siv);
    }
}

// The path of the previous player, if any.
pub fn prev_path(siv: &mut Cursive) -> Option<PathBuf> {
    let prev_path = match siv.user_data::<VecDeque<PathBuf>>() {
        Some(r) => match r.front() {
            Some(p) => Some(p.to_owned()),
            None => None,
        },
        None => None,
    };
    prev_path
}

// The path of the current player, if any.
pub fn curr_path(siv: &mut Cursive) -> Option<PathBuf> {
    let curr_path = match siv.user_data::<VecDeque<PathBuf>>() {
        Some(r) => match r.back() {
            Some(p) => Some(p.to_owned()),
            None => None,
        },
        None => None,
    };
    curr_path
}
