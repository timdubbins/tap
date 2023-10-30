mod app;
mod args;
mod data;
mod fuzzy;
mod player;
mod serialization;
mod theme;
mod utils;

fn main() {
    match app::run() {
        Ok(()) => (),
        Err(err) => eprintln!("[tap error]: {err}"),
    }
}
