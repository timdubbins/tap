mod app;
mod args;
mod data;
mod fuzzy;
mod player;
mod serialization;
mod utils;
mod views;

fn main() {
    let result: Result<(), anyhow::Error> = app::run();
    match result {
        Ok(r) => r,
        Err(err) => {
            eprintln!("[tap error]: {:#}", err);
        }
    }
}
