mod config;
mod data;
mod fuzzy;
mod player;
mod utils;

use std::path::PathBuf;

use cursive::{event::Event, CursiveRunnable};

use config::{
    args::{Args, Command},
    config::Config,
    theme,
};
use data::{persistent_data, session_data, SessionData};
use fuzzy::{FuzzyItem, FuzzyView};
use player::{PlayerBuilder, PlayerView};
use utils::IntoInner;

fn main() {
    let result = setup_and_run();

    match result {
        Ok(()) => (),
        Err(err) => eprintln!("[tap error]: {err}"),
    }
}

// Run the app.
fn setup_and_run() -> Result<(), anyhow::Error> {
    let args = Args::parse_args()?;
    let config = Config::parse_config(args);

    match config.command {
        Command::AutomatePlayer => {
            let path = fuzzy::first_audio_path(&config.path)?;
            return player::run_automated(path);
        }
        Command::SetDefault => return persistent_data::set_default_path(config.path),
        Command::PrintDefault => return persistent_data::print_default_path(),
        _ => (),
    }

    // The items to fuzzy search on.
    let items = get_items(&config)?;

    // The cursive root.
    let mut siv = cursive::ncurses();
    siv.set_theme(theme::custom(&config.colors));
    siv.set_fps(15);

    // Don't load the fuzzy-finder if there is only one audio item.
    if let Some(path) = fuzzy::only_audio_path(&config.path, &items) {
        load_standalone_player(path, &mut siv)?;
    } else {
        load_fuzzy_finder(items, &mut siv, &config.path)?;
    }

    drop(config);

    run_or_test(siv)
}

fn get_items(config: &Config) -> Result<Vec<FuzzyItem>, anyhow::Error> {
    let items =
        if config.command == Command::UseDefault || persistent_data::uses_default(&config.path) {
            persistent_data::get_cached_items(&config.path)?
        } else {
            utils::display_with_spinner(fuzzy::create_items, &config.path, "loading")?
        };

    if config.exclude_non_audio {
        Ok(fuzzy::audio_items(&items))
    } else {
        Ok(items)
    }
}

fn load_standalone_player(
    path: std::path::PathBuf,
    siv: &mut CursiveRunnable,
) -> Result<(), anyhow::Error> {
    let player = PlayerBuilder::create(path)?;
    PlayerView::load(player, siv);
    Ok(())
}

fn load_fuzzy_finder(
    items: Vec<FuzzyItem>,
    siv: &mut CursiveRunnable,
    path: &PathBuf,
) -> Result<(), anyhow::Error> {
    FuzzyView::load(items.to_owned(), None, siv);

    let session_data = SessionData::new(path, &items)?;
    siv.set_user_data(session_data.into_inner());

    siv.set_on_pre_event_inner('-', player::previous_album);
    siv.set_on_pre_event_inner('=', player::random_album);

    siv.set_on_pre_event_inner(fuzzy::trigger(), move |event: &Event| {
        fuzzy::fuzzy_finder(event, &items)
    });

    Ok(())
}

fn run_or_test(mut siv: CursiveRunnable) -> Result<(), anyhow::Error> {
    // Exit the process in test builds.
    #[cfg(feature = "run_tests")]
    {
        match siv.user_data::<InnerType<UserData>>() {
            // Output user data as stderr, if available.
            Some(user_data) => bail!("{:?}", user_data),
            None => Ok(()),
        }
    }

    // Run the Cursive event loop in non-test builds.
    #[cfg(not(feature = "run_tests"))]
    {
        siv.run();
        Ok(())
    }
}
