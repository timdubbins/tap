use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::bail;
use async_std::task;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::audio_file::AudioFile;

const FORMATS: &'static [&'static str] = &["aac", "flac", "mp3", "m4a", "ogg", "wav", "wma"];

#[derive(PartialEq)]
pub enum PlayerStatus {
    Paused,
    Playing,
    Stopped,
}

pub struct Player {
    pub path: PathBuf,
    pub playlist: Vec<AudioFile>,
    pub file: AudioFile,
    pub index: usize,
    pub is_muted: bool,
    pub status: PlayerStatus,
    pub numbers_pressed: Vec<usize>,
    pub previous_key: Arc<AtomicBool>,
    indices: HashMap<u32, usize>,
    last_started: Instant,
    last_elapsed: Duration,
    sink: Sink,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
}

pub struct Size(pub usize, pub usize);

impl Player {
    pub fn new(path: PathBuf) -> Result<(Self, Size), anyhow::Error> {
        let (playlist, x) = Player::create_playlist(path.clone())?;
        let y = std::cmp::min(45, playlist.len() + 3);
        let file = playlist
            .first()
            .expect("playlist should not be empty")
            .clone();
        let (_stream, _stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&_stream_handle)?;
        let mut indices = HashMap::new();

        for (i, f) in playlist.iter().enumerate() {
            indices.insert(f.track, i);
        }

        let mut player = Self {
            path: path,
            status: PlayerStatus::Stopped,
            last_started: Instant::now(),
            last_elapsed: Duration::default(),
            index: 0,
            numbers_pressed: vec![],
            previous_key: Arc::new(AtomicBool::new(false)),
            is_muted: false,
            playlist,
            file,
            indices,
            sink,
            _stream,
            _stream_handle,
        };

        player.play_or_pause();

        Ok((player, Size(x, y)))
    }

    pub fn play_or_pause(&mut self) {
        self.clear();

        match self.status {
            PlayerStatus::Paused => {
                self.sink.play();
                self.status = PlayerStatus::Playing;
                self.last_started = Instant::now();
            }

            PlayerStatus::Playing => {
                self.last_elapsed = self.elapsed();
                self.sink.pause();
                self.status = PlayerStatus::Paused;
            }

            // Panics on error as we currently don't have a good way to
            // handle these errors. Most of these errors *should*
            // be caught when creating the playlist.
            PlayerStatus::Stopped => {
                let p = &self.file.path;
                let f = match File::open(p.as_path()) {
                    Ok(f) => f,
                    Err(_) => panic!("Could not open '{}'.", p.display()),
                };
                let s = match Decoder::new(BufReader::new(f)) {
                    Ok(s) => s,
                    Err(_) => panic!("Could not decode '{}'.", p.display()),
                };

                self.sink.append(s);
                self.sink.play();
                self.status = PlayerStatus::Playing;
                self.last_started = Instant::now();
            }
        }
    }

    pub fn stop(&mut self) {
        self.clear();

        match self.status {
            PlayerStatus::Stopped => {}
            _ => {
                self.sink.stop();
                self.status = PlayerStatus::Stopped;
                self.last_elapsed = Duration::default()
            }
        }
    }

    pub fn play_selection(&mut self) {
        if self.select_track() {
            self.play_or_pause()
        }
    }

    pub fn play_last_track(&mut self) {
        if self.select_last_track() {
            self.play_or_pause();
        }
    }

    pub fn play_first_track(&mut self) {
        if self.select_first_track() {
            self.play_or_pause()
        }
    }

    fn clear(&mut self) {
        self.numbers_pressed.clear();
        self.previous_key.store(false, Ordering::Relaxed)
    }

    fn select_track(&mut self) -> bool {
        match self.numbers_pressed.is_empty() {
            true => {
                if self.previous_key.load(Ordering::Relaxed) {
                    self.select_first_track()
                } else {
                    // Set `previous_key` to true temporarily so that calling
                    // this function twice in quick succession will allow
                    // us to run the 'if' block of this conditional. This
                    // is to model a double tap gesture.
                    self.previous_key.store(true, Ordering::Relaxed);
                    let _previous_key = self.previous_key.clone();
                    task::spawn(async move {
                        task::sleep(Duration::from_millis(500)).await;
                        _previous_key.store(false, Ordering::Relaxed)
                    });
                    false
                }
            }
            false => self.select_track_number(),
        }
    }

    fn select_track_number(&mut self) -> bool {
        // The `numbers_pressed` array concatenated to a single value, i.e. `[0, 1, 2]` -> `12`.
        let track_number = self.numbers_pressed.iter().fold(0, |acc, x| acc * 10 + x) as u32;

        match self.indices.get(&track_number) {
            Some(i) => {
                let index = i.clone() as usize;
                self.stop();
                self.index = index;
                self.file = self.playlist[self.index].clone();
                self.clear();
                true
            }
            None => {
                self.clear();
                false
            }
        }
    }

    fn select_first_track(&mut self) -> bool {
        self.stop();
        self.index = 0;
        self.file = self.playlist[self.index].clone();
        self.clear();
        true
    }

    fn select_last_track(&mut self) -> bool {
        self.stop();
        self.index = self.playlist.len() - 1;
        self.file = self.playlist[self.index].clone();
        self.clear();
        true
    }

    pub fn next(&mut self) {
        self.clear();

        if self.index < self.playlist.len() - 1 {
            self.index += 1;
            self.file = self.playlist[self.index].clone();
        }

        match self.status {
            PlayerStatus::Stopped => self.stop(),
            _ => {
                self.stop();
                self.play_or_pause()
            }
        }
    }

    pub fn prev(&mut self) {
        self.clear();

        if self.index > 0 {
            self.index -= 1;
            self.file = self.playlist[self.index].clone();
        }

        match self.status {
            PlayerStatus::Stopped => self.stop(),
            _ => {
                self.stop();
                self.play_or_pause()
            }
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.last_elapsed
            + if self.status == PlayerStatus::Playing {
                Instant::now() - self.last_started
            } else {
                Duration::default()
            }
    }

    pub fn poll_sink(&mut self) {
        if self.status == PlayerStatus::Playing && self.sink.empty() {
            if self.index < self.playlist.len() - 1 {
                self.next();
                self.last_elapsed = Duration::default();
            } else {
                self.stop();
                self.next();
            }
        }
    }

    pub fn toggle_mute(&mut self) {
        if self.sink.volume() == 0.0 {
            self.sink.set_volume(1.0);
            self.is_muted = false;
        } else {
            self.sink.set_volume(0.0);
            self.is_muted = true;
        }
    }

    // Tries to create a playlist from the given path. Returns the list
    // and the required width for the player on success.
    pub fn create_playlist(path: PathBuf) -> Result<(Vec<AudioFile>, usize), anyhow::Error> {
        // The list of files to use in the player.
        let mut audio_files = vec![];

        // The width of the player view.
        let mut width = 0;

        // The state of the current directory.
        let mut dir_empty = true;

        // The error we get if we can't create an audio file.
        let mut error: Option<anyhow::Error> = None;

        // Add valid files to the audio_files list, updating the
        // width value on each entry. If we find a directory use
        // it to build the list instead of the current path.
        if path.is_dir() {
            for entry in path.read_dir()? {
                dir_empty = false;
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        // Recurse into child directory.
                        return Player::create_playlist(path);
                    } else if valid_ext(&path) {
                        match AudioFile::new(path) {
                            // Grow the playlist and update width.
                            Ok(f) => {
                                width = std::cmp::max(
                                    f.title.len() + 19,
                                    f.artist.len() + f.album.len() + 20,
                                );
                                audio_files.push(f)
                            }
                            // Save the error in case the playlist is empty.
                            Err(e) => error = Some(e),
                        }
                    }
                }
            }
        } else if valid_ext(&path) {
            dir_empty = false;
            match AudioFile::new(path.clone()) {
                // Create the playlist that contains a single file.
                Ok(f) => {
                    width = std::cmp::max(f.title.len() + 19, f.artist.len() + f.album.len() + 20);
                    audio_files.push(f)
                }
                // We cannot recover if the audio file is not created.
                Err(e) => bail!(e),
            }
        }

        // Give an appropriate error if we fail to
        // find any valid files.
        if audio_files.is_empty() {
            match dir_empty {
                true => bail!("'{}' is empty.", path.display()),
                false => match error {
                    Some(e) => bail!(e),
                    None => bail!("No valid files found in '{}'.", path.display()),
                },
            }
        }

        can_decode(&audio_files.first())?;
        audio_files.sort();

        Ok((audio_files, width))
    }
}

fn valid_ext(p: &PathBuf) -> bool {
    let ext = p.extension().unwrap_or_default().to_str().unwrap();
    FORMATS.contains(&ext)
}

// Returns `Ok` if the first file in audio_files can be decoded.
fn can_decode(audio_files: &Option<&AudioFile>) -> Result<(), anyhow::Error> {
    let path = audio_files.expect("audio_files not empty").path.as_path();
    let f = match File::open(path) {
        Ok(f) => f,
        Err(_) => bail!("Could not open '{}'.", path.display()),
    };
    let _ = match Decoder::new(BufReader::new(f)) {
        Ok(s) => s,
        Err(_) => bail!("Could not decode '{}'.", path.display()),
    };

    Ok(())
}
