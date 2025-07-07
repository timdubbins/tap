mod cli;
mod config;
mod finder;
mod player;

use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

use {
    anyhow::{anyhow, bail},
    colored::Colorize,
    cursive::{
        event::{Event, EventTrigger, Key, MouseButton, MouseEvent},
        theme::Theme as CursiveTheme,
        CbSink, Cursive,
    },
};

use crate::{
    cli::{player::CliPlayer, Cli},
    config::Config,
    finder::{FinderView, FuzzyDir, Library, LibraryEvent, LibraryFilter, ID as F_ID},
    player::{Player, PlayerView, Playlist, ID as P_ID},
};

pub type TapError = anyhow::Error;

pub const FPS: u32 = 1;

fn main() {
    if let Err(err) = set_up_run() {
        let err_prefix = "[tap error]:".red().bold();
        eprintln!("\r{err_prefix} {err}");
    }
}

fn set_up_run() -> Result<(), TapError> {
    let config = Config::parse_config()?;

    if config.check_version {
        return Cli::check_version();
    }

    if config.print_default_path {
        return Cli::print_cache();
    }

    if config.set_default_path {
        return Cli::set_cache(&config.search_root);
    }

    if config.use_cli_player {
        return CliPlayer::try_run(&config.search_root);
    }

    let mut siv = cursive::crossterm();
    siv.set_theme(CursiveTheme::from(&config.theme));

    if let Ok(playlist) = Playlist::process(&config.search_root, true) {
        let player = Player::try_new(playlist)?;
        PlayerView::load(&mut siv, player);
        siv.run();
    } else {
        let cb_sink = siv.cb_sink().clone();
        let (lib_tx, lib_rx) = mpsc::channel();
        let (err_tx, err_rx) = mpsc::channel::<TapError>();
        let err_state = Arc::new(Mutex::new(None));
        Library::load_in_background(&config, lib_tx.clone());
        spawn_tui_loader(lib_rx, err_tx.clone(), cb_sink.clone());
        let err_handle = spawn_err_handle(err_rx, Arc::clone(&err_state), cb_sink);
        siv.run();
        check_err_state(err_handle, err_state)?;
    }

    Ok(())
}

fn check_err_state(
    err_handle: JoinHandle<()>,
    err_state: Arc<Mutex<Option<TapError>>>,
) -> Result<(), TapError> {
    // Drop handle if we cannot join in order to prevent hanging (which occurs
    // when the event loop is quit before the UI has finished updating).
    if err_handle.is_finished() {
        err_handle.join().expect("Couldn't join err_handle")
    } else {
        drop(err_handle);
    }

    let mut err_state = err_state
        .lock()
        .map_err(|e| anyhow::anyhow!("Mutex error: {:?}", e))?;

    if let Some(err) = err_state.take() {
        bail!(err);
    }

    Ok(())
}

fn spawn_tui_loader(lib_rx: Receiver<LibraryEvent>, err_tx: Sender<TapError>, cb_sink: CbSink) {
    thread::spawn(move || {
        while let Ok(event) = lib_rx.recv() {
            let is_finished = match &event {
                LibraryEvent::Finished(_) => true,
                _ => false,
            };

            let err_tx = err_tx.clone();

            let cb = Box::new(move |siv: &mut Cursive| match event {
                LibraryEvent::Init(library) => match library.audio_count() {
                    0 => _ = err_tx.send(anyhow!("No audio found: {:?}", library.root)),
                    1 => init_player_view(library, siv, err_tx),
                    _ => init_finder_view(library, siv, err_tx),
                },
                LibraryEvent::Batch(mut batch) => update_library(&mut batch, siv),
                LibraryEvent::Finished(opt_library) => {
                    set_library(opt_library, siv, err_tx.clone());
                    set_global_callbacks(siv);
                    check_tui(siv, err_tx);
                }
            });

            _ = cb_sink.send(cb);

            if is_finished {
                break;
            }
        }
    });
}

fn init_player_view(library: Library, siv: &mut Cursive, err_tx: Sender<TapError>) {
    library
        .first_playlist()
        .and_then(Player::try_new)
        .map(|player| PlayerView::load(siv, player))
        .unwrap_or_else(|err| _ = err_tx.send(anyhow!(err)))
}

fn init_finder_view(library: Library, siv: &mut Cursive, err_tx: Sender<TapError>) {
    siv.set_user_data(library);
    FinderView::load(LibraryFilter::Unfiltered)
        .map(|e| e.process(siv))
        .unwrap_or_else(|| _ = err_tx.send(anyhow!("Failed to load library")));
    siv.call_on_name(F_ID, |fv: &mut FinderView| {
        fv.set_init_timestamp(Some(Instant::now()))
    });
}

fn update_library(batch: &mut Vec<FuzzyDir>, siv: &mut Cursive) {
    siv.with_user_data(|library: &mut Library| {
        library.fdirs.extend(batch.iter().cloned());
    });
    siv.call_on_name(F_ID, |fv: &mut FinderView| {
        fv.update_library(batch);
    });
}

fn set_library(full_library: Option<Library>, siv: &mut Cursive, err_tx: Sender<TapError>) {
    if let Some(full_library) = full_library {
        if siv.user_data::<Library>().is_some() {
            siv.set_user_data(full_library.clone());
            siv.call_on_name(F_ID, |fv: &mut FinderView| {
                fv.set_library(full_library);
            });
        } else {
            init_finder_view(full_library, siv, err_tx);
        }
    }
    siv.call_on_name(F_ID, |fv: &mut FinderView| fv.set_init_timestamp(None));
}

fn spawn_err_handle(
    err_rx: Receiver<TapError>,
    err_state: Arc<Mutex<Option<TapError>>>,
    cb_sink: CbSink,
) -> JoinHandle<()> {
    thread::spawn(move || {
        if let Ok(err) = err_rx.recv() {
            *err_state.lock().unwrap() = Some(err);
            _ = cb_sink.send(Box::new(|siv: &mut Cursive| {
                siv.quit();
            }));
        }
    })
}

fn check_tui(siv: &mut Cursive, err_tx: Sender<TapError>) {
    if siv.find_name::<PlayerView>(P_ID).is_some() {
        return;
    }

    if siv.find_name::<FinderView>(F_ID).is_some() {
        return;
    }

    let path = siv
        .user_data::<Library>()
        .map(|library| library.root.display().to_string())
        .unwrap_or_else(|| "path".to_owned());

    _ = err_tx.send(anyhow!("Error loading {:?}", path));
}

fn set_global_callbacks(siv: &mut Cursive) {
    if siv.user_data::<Library>().is_none() {
        return;
    }

    siv.set_on_pre_event('-', PlayerView::previous_album);
    siv.set_on_pre_event('=', PlayerView::random_album);
    siv.set_on_pre_event_inner(unfiltered_trigger(), FinderView::all);
    siv.set_on_pre_event_inner(fn_keys(), FinderView::depth);
    siv.set_on_pre_event_inner(uppercase_chars(), FinderView::key);
    siv.set_on_pre_event_inner(Event::CtrlChar('a'), FinderView::artist);
    siv.set_on_pre_event_inner(Event::CtrlChar('d'), FinderView::album);
    siv.set_on_pre_event_inner(Event::Char('`'), FinderView::parent);
    siv.set_on_pre_event(Event::CtrlChar('o'), open_file_manager);
    siv.set_on_pre_event(Event::CtrlChar('q'), |siv| siv.quit());
}

fn fn_keys() -> EventTrigger {
    EventTrigger::from_fn(|event| {
        matches!(
            event,
            Event::Key(Key::F1) | Event::Key(Key::F2) | Event::Key(Key::F3) | Event::Key(Key::F4)
        )
    })
}

fn uppercase_chars() -> EventTrigger {
    EventTrigger::from_fn(|event| matches!(event, Event::Char('A'..='Z')))
}

fn unfiltered_trigger() -> EventTrigger {
    EventTrigger::from_fn(|event| {
        matches!(
            event,
            Event::Key(Key::Tab)
                | Event::Mouse {
                    event: MouseEvent::Press(MouseButton::Middle),
                    ..
                }
        )
    })
}

fn open_file_manager(siv: &mut Cursive) {
    let opt_path = siv
        .call_on_name(P_ID, |pv: &mut PlayerView| {
            Some(pv.current_dir().path.clone())
        })
        .or_else(|| {
            siv.call_on_name(F_ID, |fv: &mut FinderView| {
                fv.selected_dir().map(|dir| dir.path)
            })
        })
        .flatten();

    let arg = match opt_path.as_ref().map(|p| p.as_os_str().to_str()).flatten() {
        Some(p) => p,
        None => return,
    };

    let command = match std::env::consts::OS {
        "macos" => "open",
        "linux" => "xdg-open",
        _ => return,
    };

    _ = std::process::Command::new(command).arg(arg).status();
}
