mod app;
mod args;
mod data;
mod fuzzy;
mod player;
mod serde;
mod theme;
mod utils;

fn main() {
    let result = app::run();

    match result {
        Ok(()) => (),
        Err(err) => eprintln!("[tap error]: {err}"),
    }
}
