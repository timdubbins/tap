use std::cmp::max;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::bail;
use async_std::task;
use cursive::XY;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::audio_file::{is_valid, AudioFile};
use crate::utils::concatenate;

#[derive(PartialEq)]
pub enum PlayerStatus {
    Paused,
    Playing,
    Stopped,
}

pub struct Player {
    // The path used to create the playlist.
    pub path: PathBuf,
    // The list of audio files for the player.
    pub playlist: Vec<AudioFile>,
    // The current audio file.
    pub file: AudioFile,
    // The index of the current audio file.
    pub index: usize,
    // Whether the player is muted or not.
    pub is_muted: bool,
    // Whether the player is playing, paused or stopped.
    pub status: PlayerStatus,
    // The list of numbers from last keyboard input,
    pub number_keys: Vec<usize>,
    // Whether or not a double-tap event was registered.
    pub previous_key: Arc<AtomicBool>,
    // The map of audio track numbers to file indices.
    indices: HashMap<u32, usize>,
    // The instant that playback started or resumed.
    last_started: Instant,
    // The instant that the player was paused. Reset when player is stopped.
    last_elapsed: Duration,
    // Handle to audio sink.
    sink: Sink,
    // The open flow of audio data.
    _stream: OutputStream,
    // Handle to stream.
    _stream_handle: OutputStreamHandle,
}

impl Player {
    pub fn new(path: PathBuf) -> Result<(Self, XY<usize>), anyhow::Error> {
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
            path,
            status: PlayerStatus::Stopped,
            last_started: Instant::now(),
            last_elapsed: Duration::default(),
            index: 0,
            number_keys: vec![],
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

        Ok((player, XY { x, y }))
    }

    // Whether the player is playing or not.
    fn is_playing(&self) -> bool {
        self.status == PlayerStatus::Playing
    }

    pub fn resume(&mut self) {
        self.sink.play();
        self.status = PlayerStatus::Playing;
        self.last_started = Instant::now();
    }

    pub fn pause(&mut self) {
        self.last_elapsed = self.elapsed();
        self.sink.pause();
        self.status = PlayerStatus::Paused;
    }

    pub fn play(&mut self) {
        let p = &self.file.path;
        let f = match File::open(p.as_path()) {
            Ok(f) => f,
            Err(_) => panic!("Could not open '{}'", p.display()),
        };
        let s = match Decoder::new(BufReader::new(f)) {
            Ok(s) => s,
            Err(_) => panic!("Could not decode '{}", p.display()),
        };

        self.sink.append(s);
        self.sink.play();
        self.status = PlayerStatus::Playing;
        self.last_started = Instant::now();
    }

    pub fn play_or_pause(&mut self) {
        self.clear();

        match self.status {
            PlayerStatus::Paused => self.resume(),
            PlayerStatus::Playing => self.pause(),
            PlayerStatus::Stopped => self.play(),
        };
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

    // Removes the stored keyboard inputs.
    fn clear(&mut self) {
        self.number_keys.clear();
        self.previous_key.store(false, Ordering::Relaxed)
    }

    // Selects a track to play based on stored keyboard input.
    // Returns true if a track was selected.
    fn select_track(&mut self) -> bool {
        if self.number_keys.is_empty() {
            self.select_track_double_tap()
        } else {
            self.select_track_number()
        }
    }

    // Selects the first track when called twice in quick succession.
    // This is to model a double tap gesture.
    fn select_track_double_tap(&mut self) -> bool {
        if self.previous_key.load(Ordering::Relaxed) {
            self.select_first_track()
        } else {
            // Set `previous_key` to true temporarily to gain access
            // to the 'if' block of this conditional.
            self.previous_key.store(true, Ordering::Relaxed);
            let _previous_key = self.previous_key.clone();
            task::spawn(async move {
                task::sleep(Duration::from_millis(500)).await;
                _previous_key.store(false, Ordering::Relaxed)
            });
            false
        }
    }

    pub fn select_track_index(&mut self, index: usize) {
        self.stop();
        self.index = index;
        self.file = self.playlist[self.index].clone();
        self.clear();
        self.play();
    }

    // Select the track to play from the stored keyboard input.
    fn select_track_number(&mut self) -> bool {
        let track_number = concatenate(&self.number_keys) as u32;

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

    // The time elapsed during playback.
    pub fn elapsed(&self) -> Duration {
        self.last_elapsed
            + if self.is_playing() {
                Instant::now() - self.last_started
            } else {
                Duration::default()
            }
    }

    pub fn poll_sink(&mut self) {
        if self.is_playing() && self.sink.empty() {
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
        // True when `path` is not a valid file or contains no valid files.
        let mut is_empty = true;
        // The error we get if we can't create an audio file.
        let mut error: Option<anyhow::Error> = None;

        if path.is_dir() {
            for entry in path.read_dir()? {
                is_empty = false;
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        // Recurse into child directory.
                        return Player::create_playlist(path);
                    } else if is_valid(&path) {
                        match AudioFile::new(path) {
                            // Grow the playlist and update width.
                            Ok(f) => {
                                width = max(width, f.title.len());
                                audio_files.push(f)
                            }
                            // Save the error in case the playlist is empty.
                            Err(e) => error = Some(e),
                        }
                    }
                }
            }
        } else if is_valid(&path) {
            match AudioFile::new(path.clone()) {
                // Create the playlist that contains a single file.
                Ok(f) => {
                    width = f.title.len();
                    audio_files.push(f)
                }
                // We cannot recover if the audio file is not created.
                Err(e) => bail!(e),
            }
        }

        match audio_files.first() {
            Some(f) => {
                width = max(width, f.album.len() + f.artist.len() + 1);
                can_decode(f)?;
            }
            // Give an appropriate error if we fail to find any valid files.
            None => match is_empty {
                true => bail!("'{}' is empty.", path.display()),
                false => match error {
                    Some(e) => bail!(e),
                    None => bail!("No valid files found in '{}'.", path.display()),
                },
            },
        }

        audio_files.sort();

        Ok((audio_files, width + 19))
    }
}

// Returns `Ok` if the file can be decoded.
fn can_decode(audio_file: &AudioFile) -> Result<(), anyhow::Error> {
    let path = audio_file.path.as_path();
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
