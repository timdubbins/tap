use std::io::Error;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use cursive::event::{Event, Key};
use cursive::view::Resizable;
use cursive::Cursive;

use crate::args::Args;
use crate::player::Player;
use crate::player_view::PlayerView;
use crate::search::{search_arg, SearchDir, SearchMode};

#[derive(Clone)]
pub struct App {
    pub search_dir: SearchDir,
    pub search_mode: SearchMode,
    pub initial_path: String,
    pub path: PathBuf,
    is_first_run: bool,
}

impl App {
    fn try_new() -> Result<Self, Error> {
        let (path, initial_path) = Args::parse_path_args()?;
        let (search_mode, search_dir) = Args::parse_search_options(&path);

        let app = Self {
            is_first_run: Args::parse_first_run(),
            path: path,
            initial_path: initial_path,
            search_dir: search_dir,
            search_mode: search_mode,
        };

        Ok(app)
    }

    pub fn run() -> Result<(), Error> {
        let app = App::try_new()?;

        if app.is_first_run {
            app.restart();
            return Ok(());
        }

        let (player, size) = Player::new(app.path.clone());
        let mut cursive = cursive::default();

        cursive
            .load_toml(include_str!("assets/style.toml"))
            .unwrap();

        cursive.add_layer(PlayerView::new(player).full_width().fixed_height(size));

        cursive.set_on_pre_event(Event::Char('q'), quit);
        cursive.set_on_pre_event(Event::Key(Key::Tab), move |c: &mut Cursive| {
            app.new_fuzzy_search(c)
        });
        cursive.set_fps(16);
        cursive.run();

        clear_terminal()?;
        Ok(())
    }

    fn restart(&self) {
        Command::new("/bin/bash")
            .arg("-c")
            .arg(search_arg(self))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    fn new_fuzzy_search(&self, c: &mut Cursive) {
        if self.search_mode == SearchMode::Fuzzy {
            c.pop_layer();
            self.restart();
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
