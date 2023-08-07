use std::collections::VecDeque;
use std::path::PathBuf;

use cursive::event::{Event, EventResult, EventTrigger, Key, MouseButton, MouseEvent};
use cursive::Cursive;

use crate::args::Args;
use crate::fuzzy::*;
use crate::fuzzy_view::FuzzyView;
use crate::player::Player;
use crate::player_view::PlayerView;
use crate::utils::*;

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

        // Set the refresh rate to a value that gives a steady tick.
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

        // Register the inner callbacks for fuzzy matching, filtered search, sorted
        // search, selecting the previous player and selecting a random player.
        siv.set_on_pre_event_inner(trigger(), move |event: &Event| {
            let key = inner_cb(event).expect("trigger ensures chars only");

            let items = match key.is_alphabetic() {
                true => filtered_items(key, items.to_owned()),
                false => match key.eq(&'3') {
                    true => sorted_items(items.to_owned()),
                    false => items.to_owned(),
                },
            };

            Some(EventResult::with_cb(move |siv| {
                if key.eq(&'1') {
                    random_selection(&items, siv)
                } else if key.eq(&'2') {
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

// Whether or not an inner callback is triggered.
fn trigger() -> EventTrigger {
    EventTrigger::from_fn(|event| inner_cb(event).is_some())
}

// Keybindings for the inner callbacks.
fn inner_cb(event: &Event) -> Option<char> {
    match event {
        // '0' -> fuzzy search
        Event::Key(Key::Tab)
        | Event::Mouse {
            event: MouseEvent::Press(MouseButton::Middle),
            ..
        } => Some('0'),
        Event::Char(c) => match event {
            // '1' -> random selection
            Event::Char('=') => Some('1'),
            // '2' -> previous selection
            Event::Char('-') => Some('2'),
            // 'A'..='Z' -> filtered search
            Event::Char('A'..='Z') => Some(*c),
            _ => None,
        },
        // '3' -> sorted search
        Event::CtrlChar('s') => Some('3'),
        _ => None,
    }
}

// Selects a child directory at random. Loads a new player
// with a valid selection. Invalid selections are ignored.
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

// Remove all layers from the StackView except the top layer.
pub fn remove_layers_to_top(siv: &mut Cursive) {
    let mut count = siv.screen().len();

    while count > 1 {
        siv.screen_mut()
            .remove_layer(cursive::views::LayerPosition::FromBack(0));
        count -= 1;
    }
}

// Pop all layers from the StackView except the bottom layer.
pub fn pop_layers_to_bottom(siv: &mut Cursive) {
    let mut count = siv.screen().len();

    while count > 1 {
        siv.pop_layer();
        count -= 1;
    }
}
