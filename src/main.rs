mod app;
mod args;
mod data;
mod fuzzy;
mod player;
mod serialization;
mod theme;
mod utils;

fn main() {
    let result = app::run();

    cfg_if::cfg_if! {
        if #[cfg(features = "run_tests")] {
            match result {
                Ok(_) => eprintln!("success"),
                Err(err) => eprintln!("{err}"),
            }
        } else {
            match result {
                Ok(()) => (),
                Err(err) => eprintln!("[tap error]: {err}"),
            }
        }
    }
}
