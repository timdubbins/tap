use std::env::current_exe;
use std::io::Error;
use std::process::{Command, ExitStatus};

use cursive::event::{Event, Key::Tab};
use cursive::view::Resizable;
use cursive::Cursive;

use crate::args::Args;
use crate::player::Player;
use crate::player_view::PlayerView;

mod args;
mod audio_file;
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
    let (player, size) = Player::new(Args::parse_args()?);
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
    cursive.set_on_pre_event(Event::Key(Tab), restart);
    cursive.set_fps(16);
    cursive.run();

    clear_terminal()?;
    Ok(())
}

fn check_fuzzy_command() -> bool {
    let fd_cmd = Command::new("/bin/bash")
        .arg("-c")
        .arg("fd")
        .status()
        .expect("failed to run fd");

    let fzf_cmd = Command::new("/bin/bash")
        .arg("-c")
        .arg("fzf")
        .status()
        .expect("failed to run fzf");

    fd_cmd.success() && fzf_cmd.success()
}

fn restart(c: &mut Cursive) {
    c.pop_layer();

    let arg_string = format!(
        "{} {}",
        current_exe().unwrap().display(),
        "\"$(fd -t d | fzf)\""
    );

    Command::new("/bin/bash")
        .arg("-c")
        .arg(arg_string)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    c.quit();
}

fn clear_terminal() -> Result<ExitStatus, Error> {
    Command::new("cls")
        .status()
        .or_else(|_| Command::new("clear").status())
}

fn quit(c: &mut Cursive) {
    c.quit();
}
