use std::io::Error;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use cursive::event::{Event, Key};
use cursive::view::{Nameable, Resizable};
use cursive::Cursive;

use crate::args::Args;
use crate::player::Player;
use crate::player_view::PlayerView;
use crate::search::{build_arg, SearchDir, SearchMode};
use crate::utils::path_as_string;

#[derive(Clone)]
pub struct App {
    pub search_dir: SearchDir,
    pub search_mode: SearchMode,
    pub initial_path: String,
    pub path: PathBuf,
    needs_restart: bool,
}

impl App {
    fn try_new() -> Result<Self, anyhow::Error> {
        let (path, initial_path) = Args::parse_path_args()?;
        let (search_mode, search_dir) = Args::parse_search_options(&path)?;
        let needs_restart = search_mode == SearchMode::Fuzzy && Args::is_first_run();

        let app = Self {
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
            app.restart_with_fuzzy_query();
            return Ok(());
        }

        // Without this check a playlist can be created when escaping
        // a fuzzy search. Instead we exit the program gracefully.
        if app.search_mode == SearchMode::Fuzzy
            && app.search_dir == SearchDir::PathArg
            && path_as_string(&app.path) == app.initial_path
        {
            return Ok(());
        }

        let (player, size) = Player::new(app.path.clone())?;
        let mut cursive = cursive::default();

        cursive
            .load_toml(include_str!("assets/style.toml"))
            .unwrap();

        cursive.add_layer(
            PlayerView::new(player)
                .full_width()
                .max_width(std::cmp::max(size.0, 53))
                .fixed_height(size.1)
                .with_name("player"),
        );

        cursive.set_on_pre_event(Event::Char('q'), quit);
        cursive.set_on_pre_event(Event::Key(Key::Tab), move |c: &mut Cursive| {
            app.new_fuzzy_search(c)
        });
        cursive.set_fps(16);
        cursive.run();

        clear_terminal()?;

        Ok(())
    }

    fn restart_with_fuzzy_query(&self) {
        Command::new("/bin/bash")
            .arg("-c")
            .arg(build_arg(self))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    fn new_fuzzy_search(&self, c: &mut Cursive) {
        if self.search_mode == SearchMode::Fuzzy {
            c.pop_layer();
            self.restart_with_fuzzy_query();
            c.quit()
        }
    }
}

fn clear_terminal() -> Result<ExitStatus, Error> {
    Command::new("cls")
        .status()
        .or_else(|_| Command::new("clear").status())
}

fn quit(c: &mut Cursive) {
    c.quit();
}
