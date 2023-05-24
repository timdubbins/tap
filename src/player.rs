use async_std::task;
use std::cmp;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::bail;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::audio_file::AudioFile;

const FORMATS: &'static [&'static str] = &["aac", "flac", "mp3", "mp4", "ogg", "wav"];

#[derive(PartialEq)]
pub enum PlayerStatus {
    Paused,
    Playing,
    Stopped,
}

pub struct Player {
    pub playlist: Vec<AudioFile>,
    pub file: AudioFile,
    pub index: usize,
    pub status: PlayerStatus,
    pub numbers_pressed: Vec<usize>,
    can_reach: Arc<AtomicBool>,
    last_started: Instant,
    last_elapsed: Duration,
    sink: Sink,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
}

pub struct Size(pub usize, pub usize);

impl Player {
    pub fn new(path: PathBuf) -> Result<(Self, Size), anyhow::Error> {
        let (playlist, x) = Player::create_playlist(path)?;
        let y = cmp::min(45, playlist.len() + 5);
        let file = playlist
            .first()
            .expect("playlist should not be empty")
            .clone();
        let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&_stream_handle).unwrap();

        let mut player = Self {
            status: PlayerStatus::Stopped,
            last_started: Instant::now(),
            last_elapsed: Duration::default(),
            index: 0,
            numbers_pressed: vec![],
            can_reach: Arc::new(AtomicBool::new(false)),
            playlist,
            file,
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

            PlayerStatus::Stopped => {
                let f = File::open(self.file.path.as_path()).unwrap();
                let s = Decoder::new(BufReader::new(f)).unwrap();

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

    pub fn play_last_track(&mut self) {
        self.stop();
        self.index = self.playlist.len() - 1;
        self.file = self.playlist[self.index].clone();
        self.play_or_pause();
        self.clear();
    }

    fn clear(&mut self) {
        self.numbers_pressed.clear();
        self.can_reach.store(false, Ordering::Relaxed)
    }

    pub fn select_track(&mut self) -> bool {
        match self.numbers_pressed.is_empty() {
            true => {
                if self.can_reach.load(Ordering::Relaxed) {
                    self.select_first_track()
                } else {
                    // Set `can_reach` to true temporarily so that calling
                    // this function twice in quick succession will allow
                    // us to run the 'if' block of this conditional. This
                    // is to simulate a double tap gesture.
                    self.can_reach.store(true, Ordering::Relaxed);
                    let _can_reach = self.can_reach.clone();
                    task::spawn(async move {
                        task::sleep(Duration::from_millis(500)).await;
                        _can_reach.store(false, Ordering::Relaxed)
                    });
                    false
                }
            }
            false => self.select_track_number(),
        }
    }

    fn select_track_number(&mut self) -> bool {
        // Concatenates the array of numbers, i.e. `[1, 2, 3]` -> `123`.
        let track_number = self.numbers_pressed.iter().fold(0, |acc, x| acc * 10 + x);
        let selection_valid = track_number > 0 && track_number <= self.playlist.len();

        if selection_valid {
            self.stop();
            self.index = track_number - 1;
            self.file = self.playlist[self.index].clone();
        }

        self.clear();
        selection_valid
    }

    fn select_first_track(&mut self) -> bool {
        self.stop();
        self.index = 0;
        self.file = self.playlist[self.index].clone();
        self.clear();
        true
    }

    pub fn next(&mut self) {
        self.clear();

        if self.index < self.playlist.len() - 1 {
            self.index += 1;
            self.file = self.playlist[self.index].clone();
        } else {
            self.index = 0;
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

    fn create_playlist(path: PathBuf) -> Result<(Vec<AudioFile>, usize), anyhow::Error> {
        // The list of files to use in the player.
        let mut audio_files = vec![];
        // The width of the player.
        let mut width = 0;
        // The number of entries in the current dir.
        let mut count: usize = 0;
        // The first dir we find in the current dir.
        let mut p: Option<PathBuf> = None;

        if path.is_dir() {
            for entry in path
                .read_dir()
                .expect("path is a directory has just been checked")
            {
                count += 1;
                if let Ok(entry) = entry {
                    if entry.path().is_dir() && p == None {
                        p = Some(entry.path());
                    } else if FORMATS.contains(
                        &entry
                            .path()
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap(),
                    ) {
                        if let Ok(file) = AudioFile::new(entry.path()) {
                            let next = cmp::max(
                                file.title.len() + 19,
                                file.artist.len() + file.album.len() + 20,
                            );
                            width = cmp::max(width, next);
                            audio_files.push(file)
                        }
                    }
                }
            }
        } else if FORMATS.contains(&path.extension().unwrap_or_default().to_str().unwrap()) {
            if let Ok(file) = AudioFile::new(path.clone()) {
                width = cmp::max(
                    file.title.len() + 19,
                    file.artist.len() + file.album.len() + 20,
                );
                audio_files.push(file)
            }
        }

        if audio_files.is_empty() {
            if let Some(p) = p {
                return Player::create_playlist(p);
            } else if count == 0 {
                bail!("{:?} is empty.", path)
            } else {
                bail!("no valid files found in {:?}.", path)
            }
        }

        audio_files.sort();

        Ok((audio_files, width))
    }
}
