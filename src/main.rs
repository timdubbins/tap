use std::io::Error;

use cursive::event::Event;
use cursive::view::{Resizable, Scrollable};
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

    cursive.set_on_pre_event(Event::Char('q'), |c: &mut Cursive| c.quit());
    cursive.set_fps(16);
    cursive.run();

    Ok(())
}
