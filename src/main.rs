mod app;
mod args;
mod audio_file;
mod fuzzy;
mod player;
mod serde;
mod utils;
mod views;

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
