use std::io::Error;

use cursive::event::Event;
use cursive::view::Resizable;
use cursive::Cursive;

mod args;
mod audio_file;
mod player;
mod player_status;
mod player_view;

use crate::args::Args;
use crate::player::Player;
use crate::player_view::PlayerView;

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
    let (mut player, size) = Player::new(Args::parse_args()?);
    let mut cursive = cursive::default();

    player.play_or_pause();

    cursive.add_layer(
        PlayerView::new(player, size)
            .full_width()
            .fixed_height(size),
    );

    cursive.set_on_pre_event(Event::Char('q'), |c: &mut Cursive| c.quit());
    cursive.set_fps(16);
    cursive.run();

    Ok(())
}
