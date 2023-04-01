use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::audio_file::AudioFile;
use crate::player_status::PlayerStatus;

pub struct Player {
    pub playlist: Vec<AudioFile>,
    pub file: AudioFile,
    pub index: usize,
    pub status: PlayerStatus,
    last_started: Instant,
    last_elapsed: Duration,
    sink: Sink,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
}

impl Player {
    pub fn new(path: PathBuf) -> (Self, usize) {
        let playlist = Player::create_playlist(path);
        let file = playlist
            .first()
            .expect("playlist should not be empty")
            .clone();
        let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&_stream_handle).unwrap();
        let size = playlist.len() + 3;

        let player = Self {
            status: PlayerStatus::Stopped,
            last_started: Instant::now(),
            last_elapsed: Duration::default(),
            index: 0,
            playlist,
            file,
            sink,
            _stream,
            _stream_handle,
        };

        (player, size)
    }

    pub fn play_or_pause(&mut self) {
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
        match self.status {
            PlayerStatus::Stopped => {}
            _ => {
                self.sink.stop();
                self.status = PlayerStatus::Stopped;
                self.last_elapsed = Duration::default()
            }
        }
    }

    pub fn next(&mut self) {
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
            }
        }
    }

    fn create_playlist(path: PathBuf) -> Vec<AudioFile> {
        const FORMATS: &'static [&'static str] = &["aac", "flac", "mp3", "mp4", "ogg", "wav"];

        let mut audio_files = vec![];

        if path.is_dir() {
            for entry in path.read_dir().expect("directory should not be empty") {
                if let Ok(entry) = entry {
                    if let Some(ext) = entry.path().extension() {
                        if FORMATS.contains(&ext.to_str().unwrap()) {
                            audio_files.push(AudioFile::new(entry.path()))
                        }
                    }
                }
            }
        } else if FORMATS.contains(&path.extension().unwrap_or_default().to_str().unwrap()) {
            audio_files.push(AudioFile::new(path))
        }

        if audio_files.is_empty() {
            println!("No files to play. Try a different directory.");
            std::process::exit(9);
        }

        audio_files.sort();

        audio_files
    }
}
