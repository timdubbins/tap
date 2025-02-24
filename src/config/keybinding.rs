use std::collections::HashMap;

use anyhow::bail;

use crate::TapError;

use super::FileConfig;

use {
    cursive::event::{Event, Key},
    once_cell::sync::Lazy,
};

pub struct Keybinding {}

impl Keybinding {
    fn parse(key: &str) -> Option<Event> {
        match key {
            "Up" => Some(Event::Key(Key::Up)),
            "Down" => Some(Event::Key(Key::Down)),
            "Left" => Some(Event::Key(Key::Left)),
            "Right" => Some(Event::Key(Key::Right)),
            "Enter" => Some(Event::Key(Key::Enter)),
            "Esc" => Some(Event::Key(Key::Esc)),
            "Backspace" => Some(Event::Key(Key::Backspace)),
            "Delete" => Some(Event::Key(Key::Del)),
            "Insert" => Some(Event::Key(Key::Ins)),
            "Home" => Some(Event::Key(Key::Home)),
            "End" => Some(Event::Key(Key::End)),
            "PageUp" => Some(Event::Key(Key::PageUp)),
            "PageDown" => Some(Event::Key(Key::PageDown)),
            "Tab" => Some(Event::Key(Key::Tab)),
            "Space" => Some(Event::Char(' ')),

            // Ctrl + [a..=z]
            _ if key.starts_with("Ctrl+") && key.len() == 6 => {
                key.chars().nth(5).map(Event::CtrlChar)
            }

            // All symbols and lowercased letters a..=z
            _ if key.len() == 1
                && (key.chars().next().unwrap().is_ascii_lowercase()
                    || key.chars().next().unwrap().is_ascii_punctuation()) =>
            {
                key.chars().next().map(Event::Char)
            }

            _ => None,
        }
    }

    fn default() -> HashMap<Action, Vec<Event>> {
        use Action::*;

        let mut m = HashMap::new();
        // Player Actions:
        m.insert(
            PlayOrPause,
            vec![Event::Char('h'), Event::Char(' '), Event::Key(Key::Left)],
        );
        m.insert(
            Stop,
            vec![
                Event::Char('l'),
                Event::CtrlChar('j'),
                Event::Key(Key::Enter),
                Event::Key(Key::Right),
            ],
        );
        m.insert(
            Next,
            vec![Event::Char('j'), Event::Char('n'), Event::Key(Key::Down)],
        );
        m.insert(
            Previous,
            vec![Event::Char('k'), Event::Char('p'), Event::Key(Key::Up)],
        );
        m.insert(IncreaseVolume, vec![Event::Char(']')]);
        m.insert(DecreaseVolume, vec![Event::Char('[')]);
        m.insert(ToggleMute, vec![Event::Char('m')]);
        m.insert(ToggleShowingVolume, vec![Event::Char('v')]);
        m.insert(SeekToMin, vec![Event::Char('\'')]);
        m.insert(SeekToSec, vec![Event::Char('"')]);
        m.insert(SeekForward, vec![Event::Char('.'), Event::CtrlChar('l')]);
        m.insert(SeekBackward, vec![Event::Char(','), Event::CtrlChar('h')]);
        m.insert(ToggleRandomize, vec![Event::Char('*'), Event::Char('r')]);
        m.insert(ToggleShuffle, vec![Event::Char('~'), Event::Char('s')]);
        m.insert(PlayTrackNumber, vec![Event::Char('g')]);
        m.insert(PlayLastTrack, vec![Event::CtrlChar('g'), Event::Char('e')]);
        m.insert(ShowHelp, vec![Event::Char('?')]);
        m.insert(Quit, vec![Event::Char('q')]);

        // Finder Actions:
        // m.insert(Select, vec![Event::Key(Key::Enter), Event::CtrlChar('l')]);
        // m.insert(Cancel, vec![Event::Key(Key::Esc)]);
        // m.insert(MoveDown, vec![Event::Key(Key::Down), Event::CtrlChar('n')]);
        // m.insert(MoveUp, vec![Event::Key(Key::Up), Event::CtrlChar('p')]);
        // m.insert(PageUp, vec![Event::Key(Key::PageUp), Event::CtrlChar('b')]);
        // m.insert(
        //     PageDown,
        //     vec![Event::Key(Key::PageDown), Event::CtrlChar('f')],
        // );
        // m.insert(Backspace, vec![Event::Key(Key::Backspace)]);
        // m.insert(Delete, vec![Event::Key(Key::Del)]);
        // m.insert(CursorLeft, vec![Event::Key(Key::Left)]);
        // m.insert(CursorRight, vec![Event::Key(Key::Right)]);
        // m.insert(CursorHome, vec![Event::Key(Key::Home)]);
        // m.insert(CursorEnd, vec![Event::Key(Key::End)]);
        // m.insert(ClearQuery, vec![Event::CtrlChar('u')]);
        // Global Actions:

        m
    }
}

pub static PLAYER_EVENT_TO_ACTION: Lazy<HashMap<Event, Action>> = Lazy::new(|| {
    let mut merged = Keybinding::default();

    if let Ok(config) = FileConfig::load_keybindings_only() {
        for (action_str, keys) in config.iter() {
            if let Ok(action) = Action::from_str(action_str) {
                let mut events = Vec::new();

                for key in keys {
                    if let Some(event) = Keybinding::parse(key) {
                        events.push(event);
                    } else {
                        eprintln!(
                            "[tap]: Config Warning: Invalid keybinding `{}` for action `{}`",
                            key, action_str
                        );
                    }
                }

                merged.insert(action, events);
            } else {
                eprintln!(
                    "[tap]: Config Warning: Invalid `keybinding` action `{}`",
                    action_str
                );
            }
        }
    } else {
        eprintln!("[tap]: Config Warning: Invalid `keybinding` format (if set)");
    }

    // Reverse mapping: Event → Action, for O(1) lookups.
    let mut event_map = HashMap::new();
    for (action, events) in merged {
        for event in events {
            event_map.insert(event.clone(), action);
        }
    }

    event_map
});

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Action {
    // Play Actions:
    PlayOrPause,
    Stop,
    Next,
    Previous,
    IncreaseVolume,
    DecreaseVolume,
    ToggleMute,
    ToggleShowingVolume,
    SeekToMin,
    SeekToSec,
    SeekForward,
    SeekBackward,
    ToggleRandomize,
    ToggleShuffle,
    PlayTrackNumber,
    PlayLastTrack,
    ShowHelp,
    Quit,
    // Finder Actions:
    // Select,
    // Cancel,
    // MoveDown,
    // MoveUp,
    // PageUp,
    // PageDown,
    // Backspace,
    // Delete,
    // CursorLeft,
    // CursorRight,
    // CursorHome,
    // CursorEnd,
    // ClearQuery,
    // Global Actions:
}

// Convert action name strings to `Action` enum
impl Action {
    fn from_str(action: &str) -> Result<Self, TapError> {
        use Action::*;

        match action {
            // Player Actions:
            "play_or_pause" => Ok(PlayOrPause),
            "stop" => Ok(Stop),
            "next" => Ok(Next),
            "previous" => Ok(Previous),
            "increase_volume" => Ok(IncreaseVolume),
            "decrease_volume" => Ok(DecreaseVolume),
            "toggle_mute" => Ok(ToggleMute),
            "toggle_volume_display" => Ok(ToggleShowingVolume),
            "seek_to_min" => Ok(SeekToMin),
            "seek_to_sec" => Ok(SeekToSec),
            "seek_forward" => Ok(SeekForward),
            "seek_backward" => Ok(SeekBackward),
            "toggle_randomize" => Ok(ToggleRandomize),
            "toggle_shuffle" => Ok(ToggleShuffle),
            "play_track_number" => Ok(PlayTrackNumber),
            "play_last_track" => Ok(PlayLastTrack),
            "show_help" => Ok(ShowHelp),
            "quit" => Ok(Quit),

            // Finder Actions:
            // "select" => Ok(Select),
            // "cancel" => Ok(Cancel),
            // "move_down" => Ok(MoveDown),
            // "move_up" => Ok(MoveUp),
            // "page_up" => Ok(PageUp),
            // "page_down" => Ok(PageDown),
            // "backspace" => Ok(Backspace),
            // "delete" => Ok(Delete),
            // "cursor_left" => Ok(CursorLeft),
            // "cursor_right" => Ok(CursorRight),
            // "cursor_home" => Ok(CursorHome),
            // "cursor_end" => Ok(CursorEnd),
            // "clear_query" => Ok(ClearQuery),
            // Global Actions:
            _ => bail!("Unknown action `{}`", action),
        }
    }
}
