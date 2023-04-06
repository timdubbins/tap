use std::io::Error;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use cursive::event::{Event, Key};
use cursive::view::Resizable;
use cursive::Cursive;

use crate::args::Args;
use crate::mode::Mode;
use crate::player::Player;
use crate::player_view::PlayerView;

mod args;
mod audio_file;
mod mode;
mod player;
mod player_view;

fn main() {
    let result = run();
    match result {
        Ok(r) => r,
        Err(err) => {
            eprintln!("[tap error]: {:#}", err);
        }
    }
}

fn run() -> Result<(), Error> {
    let (mode, path) = Args::parse_args()?;

    if Args::first_run() {
        mode.restart_command(path);
        return Ok(());
    }

    let (player, size) = Player::new(path.clone());
    let mut cursive = cursive::default();

    cursive
        .load_toml(include_str!("assets/style.toml"))
        .unwrap();

    cursive.add_layer(
        PlayerView::new(player, size)
            .full_width()
            .fixed_height(size), // .scrollable()
    );

    cursive.set_on_pre_event(Event::Char('q'), quit);
    cursive.set_on_pre_event(Event::Key(Key::Tab), move |c: &mut Cursive| {
        new_fuzzy_search(c, mode.clone(), path.clone())
    });
    cursive.set_fps(16);
    cursive.run();

    clear_terminal()?;
    Ok(())
}

fn new_fuzzy_search(c: &mut Cursive, mode: Mode, path: PathBuf) {
    if mode != Mode::NoFuzzy {
        c.pop_layer();
        mode.restart_command(path);
        c.quit()
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
