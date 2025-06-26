pub mod audio_file;
pub mod help_view;
pub mod player_view;
pub mod playlist;

pub use self::{
    audio_file::AudioFile, help_view::HelpView, player_view::PlayerView, playlist::Playlist,
};

use std::{
    fs::File,
    io::BufReader,
    sync::Arc,
    time::{Duration, Instant},
};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::TapError;

pub const ID: &str = "player";

// Enum representing the playback status of the audio player.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

// A struct representing an audio player that manages playback of audio files
// and maintains the state related to playback.
pub struct Player {
    // The currently loaded playlist.
    pub current: Playlist,
    // The previously loaded playlist, if any.
    pub previous: Option<Playlist>,
    // The current volume as a percentage, in range 0..=120.
    pub volume: u8,
    // Whether the player is muted or not.
    pub is_muted: bool,
    // Whether or not the next track will be selected randomly.
    pub is_randomized: bool,
    // Whether or not the current playlist order is shuffled.
    pub is_shuffled: bool,
    // Whether or not the next track is queued.
    pub next_track_queued: bool,
    // Whether the player is playing, paused or stopped.
    pub status: PlaybackStatus,
    // The instant that playback started or resumed.
    last_started: Instant,
    // The instant that the player was paused. Reset when player is stopped.
    last_elapsed: Duration,

    // Audio backend
    sink: Sink,
    stream: Arc<OutputStream>,
    stream_handle: OutputStreamHandle,
}

impl Player {
    pub fn try_new(playlist: Playlist) -> Result<Self, TapError> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        let mut player = Self {
            current: playlist,
            previous: None,
            volume: 100,
            is_muted: false,
            is_randomized: false,
            is_shuffled: false,
            next_track_queued: false,
            status: PlaybackStatus::Stopped,
            last_started: Instant::now(),
            last_elapsed: Duration::ZERO,
            stream: Arc::new(stream),
            stream_handle,
            sink,
        };

        player.play();

        Ok(player)
    }

    pub fn play(&mut self) {
        if let Ok(source) = self.current_file().decode() {
            self.sink.append(source);
            self.sink.play();
            self.status = PlaybackStatus::Playing;
            self.last_started = Instant::now();
        } else {
            self.increment_track()
        }
    }

    // Resumes a paused sink and records the start time.
    pub fn resume(&mut self) {
        self.sink.play();
        self.status = PlaybackStatus::Playing;
        self.last_started = Instant::now();
    }

    // Pauses the sink and records the elapsed time.
    pub fn pause(&mut self) {
        self.last_elapsed = self.elapsed();
        self.sink.pause();
        self.status = PlaybackStatus::Paused;
    }

    // Empties the sink, clears the current inputs and elapsed time.
    pub fn stop(&mut self) {
        self.next_track_queued = false;
        self.sink.stop();

        self.sink = rodio::Sink::try_new(&self.stream_handle).unwrap();

        self.status = PlaybackStatus::Stopped;
        self.last_elapsed = Duration::ZERO;
    }

    // Starts playback if not playing, pauses otherwise.
    pub fn play_or_pause(&mut self) {
        match self.status {
            PlaybackStatus::Paused => self.resume(),
            PlaybackStatus::Playing => self.pause(),
            PlaybackStatus::Stopped => self.play(),
        };
    }

    // Play the last track in the current playlist.
    pub fn play_last_track(&mut self) {
        let last = self.current.audio_files.len().saturating_sub(1);
        self.play_index(last);
    }

    // Skip to next track in the playlist.
    pub fn increment_track(&mut self) {
        if !self.current.is_last_track() {
            self.current.index += 1;
        }

        let is_stopped = self.is_stopped();
        self.next_track_queued = false;
        self.stop();

        if !is_stopped {
            self.play();
        }
    }

    // Skip to previous track in the playlist.
    pub fn decrement_track(&mut self) {
        let is_stopped = self.is_stopped();
        self.current.index = self.current.index.saturating_sub(1);
        self.next_track_queued = false;
        self.stop();

        if !is_stopped {
            self.play();
        }
    }

    // Increases volume by 10%, to maximum of 120%.
    pub fn increment_volume(&mut self) {
        if self.volume < 120 {
            self.volume += 10;
            if !self.is_muted {
                self.sink.set_volume(self.volume as f32 / 100.0);
            }
        }
    }

    // Decreases volume by 10%, to minimum of 0%.
    pub fn decrement_volume(&mut self) {
        if self.volume > 0 {
            self.volume -= 10;
            if !self.is_muted {
                self.sink.set_volume(self.volume as f32 / 100.0);
            }
        }
    }

    // Toggles the `is_muted` state and adjusts the volume accordingly.
    pub fn toggle_mute(&mut self) {
        self.is_muted ^= true;
        self.sink.set_volume(if self.is_muted {
            0.0
        } else {
            self.volume as f32 / 100.0
        });
    }

    // Toggles `is_randomized` and removes the current next
    // track from the sink when `is_randomized` is true.
    pub fn toggle_randomize(&mut self) {
        self.is_randomized ^= true;
        self.is_shuffled = false;
        self.next_track_queued = false;

        while self.is_randomized && self.sink.len() > 1 {
            self.sink.pop();
        }
    }

    // Toggles `is_randomized` and removes the current next
    // track from the sink when `is_randomized` is true.
    pub fn toggle_shuffle(&mut self) {
        self.is_shuffled ^= true;
        self.is_randomized = false;

        self.next_track_queued = false;

        while self.is_shuffled && self.sink.len() > 1 {
            self.sink.pop();
        }
    }

    // Seeks the playback to the provided seek_time, in seconds.
    pub fn seek_to_time(&mut self, seek_time: Duration) {
        let elapsed = self.elapsed();
        if seek_time < elapsed {
            let diff = elapsed - seek_time;
            self.seek_backward(diff);
        } else {
            let diff = seek_time - elapsed;
            self.seek_forward(diff);
        }
    }

    // Performs the seek operation in the forward direction.
    pub fn seek_forward(&mut self, seek: Duration) {
        if !self.is_playing() {
            self.play_or_pause();
        }

        let elapsed = self.elapsed();
        let duration = Duration::new(self.current_file().duration as u64, 0);

        if duration - elapsed < seek + Duration::new(0, 500) {
            if self.current.is_last_track() {
                self.stop();
            } else {
                self.increment_track()
            }
        } else {
            let future = elapsed + seek;
            if self.sink.try_seek(future).is_ok() {
                self.last_started -= seek;
            }
        }
    }

    // Performs the seek operation in the backward direction.
    pub fn seek_backward(&mut self, seek: Duration) {
        if !self.is_playing() {
            self.play_or_pause();
        }

        let elapsed = self.elapsed();

        if elapsed < seek + Duration::new(0, 500) {
            self.stop();
            self.play();
        } else {
            let time = elapsed - seek;
            if self.sink.try_seek(time).is_ok() {
                if self.last_elapsed == Duration::ZERO {
                    self.last_started += seek;
                } else if self.last_elapsed >= seek {
                    self.last_elapsed -= seek;
                } else {
                    let diff = seek - self.last_elapsed;
                    self.last_elapsed = Duration::ZERO;
                    self.last_started += diff;
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    // The time elapsed during playback.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.last_elapsed
            + if self.is_playing() {
                Instant::now() - self.last_started
            } else {
                Duration::default()
            }
    }

    // Checks and updates the playback state based on the
    // current state of the sink and the tracks queued.
    pub fn update_on_poll(&mut self) {
        if !self.is_playing() {
            return;
        }

        if self.sink.len() == 1 {
            if self.next_track_queued {
                self.last_started = Instant::now();
                self.last_elapsed = Duration::ZERO;
                self.current.index += 1;
                self.next_track_queued = false;
            } else if let Some(next) = self.current.get_next_track() {
                if let Ok(source) = next.decode() {
                    self.sink.append(source);
                    self.next_track_queued = true;
                }
            }
        } else if self.sink.empty() {
            self.stop();
        }
    }

    pub fn current_file(&self) -> &AudioFile {
        &self.current.audio_files[self.current.index]
    }

    // Whether the player is playing or not.
    pub fn is_playing(&self) -> bool {
        self.status == PlaybackStatus::Playing
    }

    // Whether the player is paused or not.
    pub fn is_paused(&self) -> bool {
        self.status == PlaybackStatus::Paused
    }

    // Whether the player is stopped or not.
    pub fn is_stopped(&self) -> bool {
        self.status == PlaybackStatus::Stopped
    }

    pub fn play_index(&mut self, index: usize) {
        self.stop();
        self.current.index = index;
        self.next_track_queued = false;
        self.play();
    }

    pub fn sink_len(&self) -> usize {
        self.sink.len()
    }

    pub fn sink_append(&self, source: Decoder<BufReader<File>>) {
        self.sink.append(source)
    }
}

impl Clone for Player {
    fn clone(&self) -> Self {
        let sink = Sink::try_new(&self.stream_handle).expect("Sink reinit");

        Player {
            current: self.current.clone(),
            previous: self.previous.clone(),
            volume: self.volume,
            is_muted: self.is_muted,
            is_randomized: self.is_randomized,
            is_shuffled: self.is_shuffled,
            next_track_queued: self.next_track_queued,
            status: self.status.clone(),
            last_started: Instant::now(),
            last_elapsed: self.last_elapsed,
            stream: self.stream.clone(),
            stream_handle: self.stream_handle.clone(),
            sink,
        }
    }
}
