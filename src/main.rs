mod app;
mod args;
mod data;
mod fuzzy;
mod player;
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
