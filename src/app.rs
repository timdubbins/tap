use std::env;
use std::path::PathBuf;

use cursive::event::{Event, Key};
use cursive::view::{Nameable, Resizable};
use cursive::Cursive;
use rand::Rng;

use crate::args::Args;
use crate::commands::*;
use crate::player::Player;
use crate::player_view::PlayerView;
use crate::utils::*;

#[derive(Clone, Copy, PartialEq)]
pub enum SearchMode {
    Fuzzy,
    NonFuzzy,
}

impl SearchMode {
    pub fn get_from(path: &PathBuf) -> Self {
        let fuzzy_available = env_var_includes(&["fzf"]) || env_var_includes(&["sk"]);
        match path_contains_dir(path) && fuzzy_available {
            true => SearchMode::Fuzzy,
            false => SearchMode::NonFuzzy,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SearchDir {
    CurrentDir,
    PathArg,
}

impl SearchDir {
    pub fn get_from(path: &PathBuf) -> Self {
        match *path == env::current_dir().unwrap() {
            true => SearchDir::CurrentDir,
            false => SearchDir::PathArg,
        }
    }
}

#[derive(Clone)]
pub struct App {
    pub search_dir: SearchDir,
    pub search_mode: SearchMode,
    pub initial_path: String,
    pub path: PathBuf,
    pub random_count: u8,
    needs_restart: bool,
}

impl App {
    fn try_new() -> Result<Self, anyhow::Error> {
        let path = Args::parse_path()?;
        let (search_mode, search_dir) = Args::parse_search_options(&path)?;
        let initial_path = Args::parse_initial_path(&path, search_mode == SearchMode::Fuzzy)?;
        let needs_restart = search_mode == SearchMode::Fuzzy && Args::is_first_run();

        let app = Self {
            random_count: 0,
            path: path,
            initial_path: initial_path,
            search_dir: search_dir,
            search_mode: search_mode,
            needs_restart: needs_restart,
        };

        Ok(app)
    }

    pub fn run() -> Result<(), anyhow::Error> {
        let app = App::try_new()?;

        // We decide whether we need fuzzy search on the first app run.
        // If we do, a restart is required in order to run the initial
        // fuzzy search.
        if app.needs_restart {
            restart_with_fuzzy_query(&app);
            return Ok(());
        }

        // Clone the app up front for use in pre-event callback.
        let app_clone = app.clone();

        // Without this check a playlist can be created when escaping
        // a fuzzy search. Instead we exit the program gracefully.
        if app.search_mode == SearchMode::Fuzzy
            && app.search_dir == SearchDir::PathArg
            && path_to_string(&app.path)? == app.initial_path
        {
            return Ok(());
        }

        let (player, size) = Player::new(app.path.clone())?;
        let mut cursive = cursive::default();

        // Set style and background color.
        cursive
            .load_toml(include_str!("assets/style.toml"))
            .unwrap();

        // Add the view for the player.
        cursive.add_layer(
            PlayerView::new(player)
                .full_width()
                .max_width(std::cmp::max(size.0, 53))
                .fixed_height(size.1)
                .with_name("player"),
        );

        // Quit the app.
        cursive.set_on_pre_event(Event::Char('q'), quit);

        // Launch new app instance with fuzzy search.
        cursive.set_on_pre_event(Event::Key(Key::Tab), move |c: &mut Cursive| {
            app_clone.new_fuzzy_search(c)
        });

        // Launch new app instance from randomized selection.
        cursive.set_on_pre_event(Event::Char('R'), move |c: &mut Cursive| {
            app.new_random_search(c);
        });

        // Set fps to lowest value that looks steady.
        cursive.set_fps(16);
        cursive.run();

        clear_terminal()?;

        Ok(())
    }

    fn new_fuzzy_search(&self, c: &mut Cursive) {
        c.pop_layer();
        restart_with_fuzzy_query(&self);
        c.quit();
    }

    fn new_random_search(&self, c: &mut Cursive) {
        let dir_count = get_dir_count(&self);
        let mut count = 0;

        let path_string: Option<String> = loop {
            if count > 10 {
                break None;
            }
            let rand = rand::thread_rng().gen_range(1..dir_count);

            let path = match get_path_string(&self, rand) {
                Some(p) => p,
                None => {
                    count += 1;
                    continue;
                }
            };
            match Player::create_playlist(PathBuf::from(&path)) {
                Ok(_) => break Some(sanitize(path)),
                Err(_) => {
                    count += 1;
                    continue;
                }
            }
        };

        if let Some(p) = path_string {
            c.pop_layer();
            restart_with_path_string(&self, p);
            c.quit();
        }
    }
}

fn quit(c: &mut Cursive) {
    c.quit();
}
