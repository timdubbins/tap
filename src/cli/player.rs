use std::{
    io::{self, stdout, BufRead, Write},
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread::{self},
    time::Duration,
};

use colored::Colorize;

use crate::{
    player::{Player, Playlist},
    TapError,
};

const TICK: Duration = Duration::from_millis(100);

// A CLI wrapper for the `Player` struct, responsible for managing the
// audio playback and displaying the current status to the terminal.
pub struct CliPlayer {
    player: Player,
}

impl CliPlayer {
    // Runs an audio player in the command line without the TUI.
    pub fn try_run(search_root: &PathBuf) -> Result<(), TapError> {
        use crate::finder::Library;

        let playlist = Library::first(search_root).first_playlist()?;
        let mut cli_player = CliPlayer::try_new(playlist)?;
        cli_player.start()
    }

    fn try_new(playlist: Playlist) -> Result<Self, TapError> {
        let player = Player::try_new(playlist)?;

        player
            .current
            .audio_files
            .iter()
            .skip(1)
            .take(100)
            .filter_map(|file| file.decode().ok())
            .for_each(|source| player.sink.append(source));

        Ok(Self { player })
    }

    fn start(&mut self) -> Result<(), TapError> {
        self.update_display()?;
        let rx = Self::setup_input_thread();
        let mut len = self.player.sink.len();

        loop {
            let current_len = self.player.sink.len();

            if Self::should_exit(&rx, current_len) {
                return Ok(());
            }

            if current_len < len {
                len = current_len;
                self.player.current.index += 1;
                self.update_display()?;
            }

            thread::sleep(TICK);
        }
    }

    fn setup_input_thread() -> Receiver<bool> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let stdin = io::stdin();
            let handle = stdin.lock();

            for line in handle.lines() {
                let line = line.expect("Failed to read line");
                if line.trim().is_empty() {
                    tx.send(true).expect("Failed to send quit signal");
                    break;
                }
            }
        });

        rx
    }

    fn should_exit(rx: &Receiver<bool>, len: usize) -> bool {
        len == 0 || rx.try_recv() != Err(TryRecvError::Empty)
    }

    fn update_display(&mut self) -> Result<(), TapError> {
        let file = self.player.current_file();
        let tap_prefix = "[tap player]:".green().bold();

        let player_info = format!(
            "{} '{}' by '{}' ({}/{}) ",
            tap_prefix,
            file.title,
            file.artist,
            self.player.current.index + 1,
            self.player.current.audio_files.len()
        );

        print!("\r\x1b[2K{}", player_info); // \x1b[2K clears the entire line
        stdout().flush()?;

        Ok(())
    }
}
