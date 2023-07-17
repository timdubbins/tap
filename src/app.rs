use std::collections::VecDeque;
use std::path::PathBuf;

use cursive::event::{Event, EventResult, Key};
use cursive::view::Resizable;
use cursive::Cursive;
use walkdir::DirEntry;

use crate::args::Args;
use crate::commands::*;
use crate::player::{Player, Size};
use crate::player_view::PlayerView;
use crate::utils::*;

#[derive(Clone)]
pub struct App {
    // The initial path, which can be either a file / directory
    // to play, or a directory to search on.
    pub path: PathBuf,
    // Whether or not path can be used to search on.
    pub searchable: bool,
    // The list of directories we can search on.
    // Empty if searchable is false.
    pub dirs: Vec<DirEntry>,
    // The list of directories we can search on, joined by the
    // newline character. Used as the input for fuzzy searching.
    // Empty if searchable is false.
    pub search_string: String,
    // The fuzzy program to use, if available.
    pub fuzzy_mode: Option<FuzzyMode>,
}

impl App {
    fn try_new() -> Result<Self, anyhow::Error> {
        let path = Args::parse_path()?;
        let searchable = has_child_dirs(&path);
        let fuzzy_mode = FuzzyMode::get(searchable);

        if searchable && fuzzy_mode.is_none() {
            anyhow::bail!(
                "'{}' contains subdirectories and requires a fuzzy-finder to run. \
                Install either `fzf` or `skim` to enable fuzzy-finding.",
                path.display()
            )
        }

        let (dirs, search_string) = match searchable {
            true => {
                let dirs = dirs(&path);
                let search_string = search_string(&dirs);
                (dirs, search_string)
            }
            false => (vec![], String::from("")),
        };

        let app = Self {
            fuzzy_mode: FuzzyMode::get(searchable),
            path,
            searchable,
            dirs,
            search_string,
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
                        app.filtered_fuzzy_search(anchor, s)
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
        // Add dummy user data so we can load initial player.
        s.set_user_data(VecDeque::from([PathBuf::new(), PathBuf::new()]));

        // Add dummy view, also required to load initial player.
        s.add_layer(cursive::views::DummyView);

        if self.fuzzy_mode.is_some() {
            self.initial_fuzzy_search(s)
        } else {
            let (player, size) = Player::new(self.path.clone())?;
            load_player((player, size), s);
        }

        // Replace the dummy user data with path of initial player.
        let curr_path = current_path(s);
        s.set_user_data(VecDeque::from([curr_path.clone(), curr_path]));

        Ok(())
    }

    // Runs a fuzzy search on all child directories.
    // Invalid selections are ignored.
    fn fuzzy_search(&self, s: &mut Cursive) {
        self._fuzzy_search(None, false, None, s)
    }

    // Runs a fuzzy search on top level directories that start
    // with the `anchor` letter. Invalid selections are ignored.
    fn filtered_fuzzy_search(&self, anchor: Option<String>, s: &mut Cursive) {
        self._fuzzy_search(None, false, anchor, s)
    }

    // Runs the initial fuzzy search.
    // Exits the program if the selection is invalid.
    fn initial_fuzzy_search(&self, s: &mut Cursive) {
        s.add_layer(cursive::views::TextView::new("").full_screen());
        self._fuzzy_search(None, true, None, s)
    }

    // Tries to load a new player from a fuzzy search. Selections that
    // contain two or more children are fuzzy searched on.
    fn _fuzzy_search(
        &self,
        second_path: Option<PathBuf>,
        is_first_run: bool,
        anchor: Option<String>,
        s: &mut Cursive,
    ) {
        if self.fuzzy_mode.is_none() {
            return;
        }

        let fuzzy_path = fuzzy_path(&self, second_path.clone(), anchor);
        let curr_path = current_path(s);

        // Try to load a new player from the fuzzy path.
        if fuzzy_path.eq(&self.path) || fuzzy_path.eq(&curr_path) {
            if is_first_run {
                // Initial fuzzy search was escaped. This
                // is not considered an error.
                std::process::exit(0);
            } else {
                // Subsequent fuzzy search was escaped. We resume the
                // current player which requires redrawing the screen.
                s.clear();
            }
        } else if has_child_dirs(&fuzzy_path) {
            // Descend the directory levels recursively until
            // the selection contains less than two children.
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
    }

    // Selects a child directory at random. Loads a new player
    // with a valid selection. Invalid selections are ignored.
    fn random_selection(&self, s: &mut Cursive) {
        if !self.searchable {
            return;
        }

        // let dir_count = get_dir_count(&self);
        let mut count = 0;

        // Loop until we find a valid selection or we give up.
        while count < 10 {
            // let random_path = get_random_path(&self, dir_count);
            let random_path = random_path(&self);
            let curr_path = current_path(s);
            if random_path.eq(&curr_path) {
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
    // search_events:
    // '0' : fuzzy_search
    // '1' : random_selection
    // '2' : previous_selection
    // 'a...z' : fuzzy_search_with_anchor
    match event {
        Event::Key(Key::Tab) => Some('0'),
        Event::Char(_) => match event {
            Event::Char('r') => Some('1'),
            Event::Char('=') => Some('1'),
            Event::Char('b') => Some('2'),
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

// Gets path of previous player from user data.
fn previous_path(s: &mut Cursive) -> PathBuf {
    let prev_path = s
        .user_data::<VecDeque<PathBuf>>()
        .expect("user data should be set on init")
        .front()
        .expect("previous path should be at the start of the queue")
        .to_owned();

    prev_path
}

// Gets path of current player from user data.
fn current_path(s: &mut Cursive) -> PathBuf {
    let curr_path = s
        .user_data::<VecDeque<PathBuf>>()
        .expect("user data should be set on init")
        .back()
        .expect("player path should be appended when player is created")
        .to_owned();

    curr_path
}

// Updates the user data and player view.
fn load_player((player, size): (Player, Size), s: &mut Cursive) {
    let path = player.path.clone();

    // Add the new player view.
    s.add_layer(
        PlayerView::new(player)
            .full_width()
            .max_width(std::cmp::max(size.0, 53))
            .fixed_height(size.1),
    );

    // Remove the previous player view.
    s.screen_mut()
        .remove_layer(cursive::views::LayerPosition::FromFront(1));

    // Keep a reference to the current and previous player.
    s.with_user_data(|history: &mut VecDeque<PathBuf>| {
        history.push_back(path);
        history.pop_front();
    });
}

// Selects and loads the previous player.
fn previous_selection(s: &mut Cursive) {
    let prev_path = previous_path(s);
    let (player, size) =
        Player::new(prev_path.clone()).expect("path has been validated with the previous player");

    load_player((player, size), s);
}

// The fuzzy-search utilities supported by tap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FuzzyMode {
    // https://github.com/junegunn/fzf
    FZF,
    // https://github.com/lotabout/skim
    SK,
}

impl FuzzyMode {
    // Gets fuzzy program that is available. Selects "fzf" if both
    // "fzf" and "sk" are installed. None if neither is installed.
    fn get(searchable: bool) -> Option<Self> {
        if searchable {
            if env_var_includes(&["fzf"]) {
                return Some(FuzzyMode::FZF);
            } else if env_var_includes(&["sk"]) {
                return Some(FuzzyMode::SK);
            }
        }
        None
    }
}
