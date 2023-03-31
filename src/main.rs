use core::cmp::Ordering;
use std::fs::File;
use std::io::{BufReader, Error};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;
use cursive::event::{Event, EventResult};
use cursive::traits::{Resizable, View};
use cursive::{Cursive, Printer};
use lofty::{Accessor, AudioFile as LoftyAudioFile, Probe, TaggedFileExt};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

#[derive(Parser)]
struct Args {
    path: Option<std::path::PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord)]
struct AudioFile {
    path: PathBuf,
    title: String,
    artist: String,
    album: String,
    year: Option<u32>,
    track: u32,
    duration: u64,
    duration_display: String,
}

impl AudioFile {
    fn new(path: PathBuf) -> Self {
        let tagged_file = Probe::open(&path)
            .expect("ERROR: Bad path provided!")
            .read()
            .expect("ERROR: Failed to read file!");

        let tag = match tagged_file.primary_tag() {
            Some(primary_tag) => primary_tag,
            None => tagged_file.first_tag().expect("ERROR: No tags found!"),
        };

        let properties = tagged_file.properties();
        let duration = properties.duration().as_secs();

        Self {
            path,
            title: tag.title().as_deref().unwrap_or("None").trim().to_string(),
            artist: tag.artist().as_deref().unwrap_or("None").trim().to_string(),
            album: tag.album().as_deref().unwrap_or("None").trim().to_string(),
            year: tag.year(),
            track: tag.track().unwrap_or(0),
            duration,
            duration_display: format!("{:02}:{:02}", duration / 60, duration % 60),
        }
    }
}

impl PartialOrd for AudioFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.artist == other.artist {
            if self.album == other.album {
                if self.track == other.track {
                    return Some(self.title.cmp(&other.title));
                }
                return Some(self.track.cmp(&other.track));
            }
            return Some(self.album.cmp(&other.album));
        }
        Some(self.artist.cmp(&other.artist))
    }
}

#[allow(dead_code)]
// #[derive(PartialEq)]
struct Player {
    file: AudioFile,
    playlist: Vec<AudioFile>,
    index: usize,
    sink: Sink,
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    status: PlayerStatus,
    last_started: Instant,
    last_elapsed: Duration,
}

impl Player {
    fn new(playlist: Vec<AudioFile>) -> Self {
        let playlist = playlist;
        let index: usize = 0;
        let file = playlist
            .first()
            .expect("playlist should not be empty")
            .clone();
        let status = PlayerStatus::Stopped;
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let last_started = Instant::now();
        let last_elapsed = Duration::default();

        Self {
            file,
            playlist,
            index,
            status,
            sink,
            stream,
            stream_handle,
            last_started,
            last_elapsed,
        }
    }

    fn play_or_pause(&mut self) {
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

    fn stop(&mut self) {
        match self.status {
            PlayerStatus::Stopped => {}
            _ => {
                self.sink.stop();
                self.status = PlayerStatus::Stopped;
                self.last_elapsed = Duration::default()
            }
        }
    }

    fn next(&mut self) {
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

    fn prev(&mut self) {
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

    fn elapsed(&self) -> Duration {
        self.last_elapsed
            + if self.status == PlayerStatus::Playing {
                Instant::now() - self.last_started
            } else {
                Duration::default()
            }
    }

    fn poll_sink(&mut self) {
        if self.status == PlayerStatus::Playing && self.sink.empty() {
            if self.index < self.playlist.len() - 1 {
                self.next();
                self.last_elapsed = Duration::default();
            } else {
                self.stop();
            }
        }
    }
}

#[derive(PartialEq)]
enum PlayerStatus {
    Paused,
    Playing,
    Stopped,
}

fn create_playlist(path: PathBuf) -> Vec<AudioFile> {
    const EXTENSIONS: &'static [&'static str] = &["aac", "flac", "mp3", "mp4", "ogg", "wav"];

    let mut audio_files = vec![];

    // let mut entries = args
    //     .path
    //     .read_dir()?
    //     .map(|res| res.map(|e| e.path()))
    //     .collect::<Result<Vec<_>, std::io::Error>>();

    if path.is_dir() {
        for entry in path.read_dir().expect("directory should not be empty") {
            if let Ok(entry) = entry {
                if let Some(ext) = entry.path().extension() {
                    if EXTENSIONS.contains(&ext.to_str().unwrap()) {
                        audio_files.push(AudioFile::new(entry.path()))
                    }
                }
            }
        }
    } else if EXTENSIONS.contains(&path.extension().unwrap().to_str().unwrap()) {
        audio_files.push(AudioFile::new(path))
    }

    if audio_files.is_empty() {
        println!("No files to play. Try a different directory.");
        std::process::exit(9);
    }

    audio_files.sort();
    audio_files
}

fn main() -> Result<(), Error> {
    let path = match Args::parse().path {
        Some(p) => p,
        None => std::env::current_dir()?,
    };
    let mut player = Player::new(create_playlist(path));
    let size = &player.playlist.len() + 3;
    let mut cursive = cursive::default();

    player.play_or_pause();
    cursive.set_on_pre_event(Event::Char('q'), |c: &mut Cursive| c.quit());
    cursive.add_layer(
        PlayerView::new(size, player)
            .full_width()
            .fixed_height(size),
    );
    cursive.set_fps(16);
    cursive.run();

    Ok(())
}

struct PlayerView {
    player: Player,
    size: usize,
}

impl PlayerView {
    fn new(size: usize, player: Player) -> Self {
        Self { player, size }
    }
}

impl View for PlayerView {
    fn draw(&self, printer: &Printer) {
        let f = &self.player.file;
        let elapsed = self.player.elapsed().as_secs();

        let header = match f.year {
            Some(y) => format!("{} - {} - {}", f.artist, f.album, y),
            None => format!("{} - {}", f.artist, f.album),
        };

        let status = match self.player.status {
            PlayerStatus::Paused => "||",
            PlayerStatus::Playing => ">",
            PlayerStatus::Stopped => ".",
        };

        printer.print((0, 0), &header.as_str());

        for (y, f) in self.player.playlist.iter().enumerate() {
            let line = format!("{:02} - {} - {}", f.track, f.title, f.duration_display);

            if y == self.player.index {
                printer.print((1, y + 1), status);
            }

            printer.print((4, y + 1), &line);
        }

        printer.print(
            (0, self.size - 1),
            &format!("{:02}:{:02}", elapsed / 60, elapsed % 60),
        );
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Refresh => {
                self.player.poll_sink();
                EventResult::Consumed(None)
            }

            Event::Char('p') => {
                self.player.play_or_pause();
                EventResult::Consumed(None)
            }

            Event::Char('s') => {
                self.player.stop();
                EventResult::Consumed(None)
            }

            Event::Char('j') => {
                self.player.next();
                EventResult::Consumed(None)
            }

            Event::Char('k') => {
                self.player.prev();
                EventResult::Consumed(None)
            }

            _ => EventResult::Consumed(None),
        }
    }
}
