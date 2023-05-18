mod app;
mod args;
mod audio_file;
mod player;
mod player_view;
mod search;
mod theme;
mod utils;

use crate::app::App;

fn main() {
    let result: Result<(), anyhow::Error> = App::run();
    match result {
        Ok(r) => r,
        Err(err) => {
            eprintln!("[tap error]: {:#}", err);
        }
    }
}
