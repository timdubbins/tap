use std::path::PathBuf;

use cursive::event::{Event, EventResult, Key};
use cursive::view::Resizable;
use cursive::Cursive;

use crate::args::Args;
use crate::commands::*;
use crate::player::{Player, Size};
use crate::player_view::PlayerView;
use crate::utils::*;

#[derive(Clone)]
pub struct App {
    pub fd_available: bool,
    pub fuzzy_mode: FuzzyMode,
    pub path: PathBuf,
    pub path_string: String,
    pub search_dir: SearchDir,
    pub searchable: bool,
}

impl App {
    fn try_new() -> Result<Self, anyhow::Error> {
        let (path, path_string) = Args::parse_path()?;
        let searchable = has_child_dirs(&path);
        let fuzzy_mode = FuzzyMode::get(searchable);

        if searchable && fuzzy_mode == FuzzyMode::None {
            anyhow::bail!(
                "'{}' contains subdirectories and requires a fuzzy-finder to run. \
                Install either `fzf` or `skim` to enable fuzzy-finding.",
                path.display()
            )
        }

        let app = Self {
            fd_available: env_var_includes(&["fd"]),
            fuzzy_mode: FuzzyMode::get(searchable),
            search_dir: SearchDir::get(&path)?,
            path: path,
            path_string: path_string,
            searchable,
        };

        Ok(app)
    }

    pub fn run() -> Result<(), anyhow::Error> {
        let app = App::try_new()?;

        // The cursive root.
        let mut siv = cursive::default();

        // Set style and background color.
        siv.load_toml(include_str!("assets/style.toml"))
            .expect("style.toml should be located in assets directory");

        // Initialize the player and player view.
        app.init_player(&mut siv)?;

        // Tries to create a new player and player view from `search_events`.
        // Replaces the old player and player view on success.
        siv.set_on_pre_event_inner(is_search_event, move |e: &Event| {
            let c = search_events(e);
            let app = app.clone();
            Some(EventResult::with_cb(move |s| {
                if let Some(c) = c {
                    if c.is_alphabetic() {
                        let anchor = Some(c.into());
                        app.fuzzy_search_with_anchor(anchor, s)
                    } else if c.eq(&'0') {
                        app.fuzzy_search(s)
                    } else if c.eq(&'1') {
                        app.random_selection(s)
                    } else if c.eq(&'2') {
                        previous_selection(s)
                    }
                }
            }))
        });

        // Quit the app.
        siv.set_on_pre_event(Event::Char('q'), |s: &mut Cursive| s.quit());

        // Set to lowest value that looks steady.
        siv.set_fps(16);

        // Start the event loop.
        siv.run();

        clear_terminal()?;
        Ok(())
    }

    fn init_player(&self, s: &mut Cursive) -> Result<(), anyhow::Error> {
        // Add dummy user data so we can load the initial player.
        s.set_user_data(vec![PathBuf::new()]);

        if self.fuzzy_mode != FuzzyMode::None {
            self.initial_fuzzy_search(s)
        } else {
            let (player, size) = Player::new(self.path.clone())?;
            load_player((player, size), s);
        }

        // Replace the dummy user data with a copy of the initial player path.
        s.with_user_data(|paths: &mut Vec<PathBuf>| {
            let p = paths.last().expect("path set on init");
            paths.push(p.clone());
            paths.remove(0);
        });

        Ok(())
    }

    // Runs a fuzzy search on all child directories. Attempts to
    // descend into child directories if the selection contains
    // subdirectories. Invalid selections are ignored.
    fn fuzzy_search(&self, s: &mut Cursive) {
        self._fuzzy_search(None, false, None, s)
    }

    // Runs a fuzzy search on top level directories that start
    // with the `anchor` letter. Runs a second fuzzy search if
    // the selection contains subdirectories. Invalid selections
    // are ignored.
    fn fuzzy_search_with_anchor(&self, anchor: Option<String>, s: &mut Cursive) {
        self._fuzzy_search(None, false, anchor, s)
    }

    // Runs the initial fuzzy search. Exits the program if
    // the selection is invalid.
    fn initial_fuzzy_search(&self, s: &mut Cursive) {
        self._fuzzy_search(None, true, None, s)
    }

    // Tries to load a new player from a fuzzy search.
    fn _fuzzy_search(
        &self,
        second_path: Option<PathBuf>,
        is_first_run: bool,
        anchor: Option<String>,
        s: &mut Cursive,
    ) {
        if self.fuzzy_mode == FuzzyMode::None {
            return;
        }

        let fuzzy_path = get_fuzzy_path(&self, second_path.clone(), anchor);
        let curr_path = s
            .user_data::<Vec<PathBuf>>()
            .expect("user data should be set on init")
            .last()
            .expect("current path is the last entry in user data");

        let mut search_root = match second_path {
            Some(p) => p,
            None => self.path.clone(),
        };
        // Push an empty path to append a trailing slash.
        search_root.push("");

        // Try to load a new player from the fuzzy path.
        if fuzzy_path.eq(&search_root) || fuzzy_path.eq(curr_path) {
            if is_first_run {
                // Initial fuzzy search was escaped. This
                // is not considered an error.
                std::process::exit(0);
            }
        } else if has_child_dirs(&fuzzy_path) {
            // The fuzzy_path contains subdirectories so we use
            // it to spawn another fuzzy search, recursing until
            // we find a leaf directory.
            if let Ok(p) = remove_trailing_slash(fuzzy_path.clone()) {
                self._fuzzy_search(Some(p), is_first_run, None, s);
                return;
            }
        } else {
            match Player::new(fuzzy_path) {
                Ok((player, size)) => {
                    load_player((player, size), s);
                }
                Err(e) => {
                    if is_first_run {
                        // The event loop has not been run so we can print
                        // the error message without returning a result.
                        eprintln!("[tap error]: {:#}", e);
                        std::process::exit(1);
                    }
                }
            }
        }

        // We are here if the new player was loaded or the old
        // player was resumed. In either case we need a redraw.
        s.clear();
    }

    // Selects a child directory at random. Loads a new player
    // with a valid selection. Invalid selections are ignored.
    fn random_selection(&self, s: &mut Cursive) {
        if !self.searchable {
            return;
        }

        let dir_count = get_dir_count(&self);
        let mut count = 0;

        // Loop until we find a valid selection or we give up.
        while count < 10 {
            let random_path = get_random_path(&self, dir_count);
            let curr_path = s
                .user_data::<Vec<PathBuf>>()
                .expect("user data should be set on init")
                .last()
                .expect("current path is the last entry in user data");

            if random_path.eq(curr_path) {
                // Don't reload the same player, try a different path.
                count += 1
            } else if let Ok((player, size)) = Player::new(random_path) {
                load_player((player, size), s);
                break;
            } else {
                count += 1;
            }
        }
    }
}

// Returns true if a `search_event` has been triggered.
fn is_search_event(event: &Event) -> bool {
    search_events(event) != None
}

// Creates a mapping of events to `search_events`. This allows
// us to match on the `search_events` from an inner callback
// without needing multiple clones of `app`.
fn search_events(event: &Event) -> Option<char> {
    // '0' : fuzzy_search
    // '1' : random_selection
    // '2' : previous_selection
    // 'a...z' : fuzzy_search_with_anchor
    match event {
        Event::Key(Key::Tab) => Some('0'),
        Event::Char(_) => match event {
            Event::Char('r') => Some('1'),
            Event::Char('-') => Some('2'),
            Event::Char('A') => Some('a'),
            Event::Char('B') => Some('b'),
            Event::Char('C') => Some('c'),
            Event::Char('D') => Some('d'),
            Event::Char('E') => Some('e'),
            Event::Char('F') => Some('f'),
            Event::Char('G') => Some('g'),
            Event::Char('H') => Some('h'),
            Event::Char('I') => Some('i'),
            Event::Char('J') => Some('j'),
            Event::Char('K') => Some('k'),
            Event::Char('L') => Some('l'),
            Event::Char('M') => Some('m'),
            Event::Char('N') => Some('n'),
            Event::Char('O') => Some('o'),
            Event::Char('P') => Some('p'),
            Event::Char('Q') => Some('q'),
            Event::Char('R') => Some('r'),
            Event::Char('S') => Some('s'),
            Event::Char('T') => Some('t'),
            Event::Char('U') => Some('u'),
            Event::Char('V') => Some('v'),
            Event::Char('W') => Some('w'),
            Event::Char('X') => Some('x'),
            Event::Char('Y') => Some('y'),
            Event::Char('Z') => Some('z'),
            _ => None,
        },
        _ => None,
    }
}

// Updates the user data and player view.
fn load_player((player, size): (Player, Size), s: &mut Cursive) {
    s.with_user_data(|paths: &mut Vec<PathBuf>| {
        paths.push(player.path.clone());
        if paths.len() > 2 {
            paths.remove(0);
        }
    });
    s.pop_layer();
    s.add_layer(
        PlayerView::new(player)
            .full_width()
            .max_width(std::cmp::max(size.0, 53))
            .fixed_height(size.1),
    );
}

// Selects and loads the previous player.
fn previous_selection(s: &mut Cursive) {
    let prev_path = s
        .user_data::<Vec<PathBuf>>()
        .expect("user data should be set on init")
        .first()
        .expect("previous path is at index 0 in user data");

    let (player, size) =
        Player::new(prev_path.clone()).expect("player created from this path previously");

    load_player((player, size), s);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FuzzyMode {
    FZF,
    SK,
    None,
}

impl FuzzyMode {
    pub fn get(searchable: bool) -> Self {
        if searchable {
            if env_var_includes(&["fzf"]) {
                return FuzzyMode::FZF;
            } else if env_var_includes(&["sk"]) {
                return FuzzyMode::SK;
            }
        }
        FuzzyMode::None
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SearchDir {
    CurrentDir,
    PathArg,
}

impl SearchDir {
    pub fn get(path: &PathBuf) -> Result<Self, anyhow::Error> {
        if std::env::current_dir()?.eq(path) {
            return Ok(SearchDir::CurrentDir);
        } else {
            return Ok(SearchDir::PathArg);
        }
    }
}
