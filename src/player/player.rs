use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::bail;
use cursive::XY;
use expiring_bool::ExpiringBool;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::args;
use crate::utils;

use super::{valid_audio_ext, AudioFile, PlayerOpts, PlayerStatus, StatusToBytes};

pub type PlayerResult = Result<(Player, bool, XY<usize>), anyhow::Error>;

const SEEK_TIME: Duration = Duration::from_secs(10);

pub struct Player {
    // The path used to create the playlist.
    pub path: PathBuf,
    // The list of audio files for the player.
    pub playlist: Vec<AudioFile>,
    // The current audio file.
    pub file: AudioFile,
    // The index of the current audio file.
    pub index: usize,
    // The index of the previous audio file, used with standalone player.
    pub previous: usize,
    // The current volume as a percentage, in range 0..=120.
    pub volume: u8,
    // Whether the player is muted or not.
    pub is_muted: bool,
    // Whether or not the next track will be selected randomly.
    pub is_randomized: bool,
    // Whether or not the next track is queued.
    pub next_track_queued: bool,
    // Whether the player is playing, paused or stopped.
    pub status: PlayerStatus,
    // The list of numbers from last keyboard input,
    pub num_keys: Vec<usize>,
    // Whether or not a double-tap event was registered.
    pub timer_bool: ExpiringBool,
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
    pub fn new(
        track: (PathBuf, usize),
        opts: PlayerOpts,
        is_randomized: bool,
        recurse: bool,
    ) -> PlayerResult {
        let (path, index) = (track.0, track.1);
        let (playlist, size) = playlist(&path, recurse)?;
        let file = playlist[index].to_owned();
        let (_stream, _stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&_stream_handle)?;
        let mut indices = HashMap::new();

        for (i, f) in playlist.iter().enumerate() {
            indices.insert(f.track, i);
        }

        let mut player = Self {
            last_started: Instant::now(),
            last_elapsed: Duration::ZERO,
            previous: 0,
            num_keys: vec![],
            next_track_queued: false,
            timer_bool: ExpiringBool::new(false, Duration::from_millis(500)),
            status: opts.status,
            volume: opts.volume,
            is_muted: opts.is_muted,
            path,
            index,
            playlist,
            file,
            indices,
            is_randomized,
            sink,
            _stream,
            _stream_handle,
        };

        player.set_volume();
        player.set_playback();

        Ok((player, opts.showing_volume, size))
    }

    // Resumes a paused sink and records the start time.
    pub fn resume(&mut self) {
        self.sink.play();
        self.status = PlayerStatus::Playing;
        self.last_started = Instant::now();
    }

    // Pauses the sink and records the elapsed time.
    pub fn pause(&mut self) {
        self.last_elapsed = self.elapsed();
        self.sink.pause();
        self.status = PlayerStatus::Paused;
    }

    // Empties the sink, clears the current inputs and elapsed time.
    pub fn stop(&mut self) -> u8 {
        self.clear();
        if self.status != PlayerStatus::Stopped {
            self.sink.stop();
            self.status = PlayerStatus::Stopped;
            self.last_elapsed = Duration::ZERO;
        }
        self.status.to_u8()
    }

    // Decodes and appends `file` to the sink, starts playback and records start time.
    pub fn play(&mut self) {
        if let Ok(source) = decode(&self.file.path) {
            self.sink.append(source);
            self.sink.play();
            self.status = PlayerStatus::Playing;
            self.last_started = Instant::now();
        } else {
            self.next()
        }
    }

    // Starts playback if not playing, pauses otherwise.
    pub fn play_or_pause(&mut self) -> u8 {
        match self.status {
            PlayerStatus::Paused => self.resume(),
            PlayerStatus::Playing => self.pause(),
            PlayerStatus::Stopped => self.play(),
        };
        self.status.to_u8()
    }

    // Play the track selected from keyboard input.
    pub fn play_key_selection(&mut self) {
        // Play first track when called in quick succession.
        if self.num_keys.is_empty() {
            if self.timer_bool.is_true() {
                self.play_index(0);
            } else {
                self.timer_bool.set();
            }
        // Play the track from number key inputs.
        } else {
            let track_number = utils::concatenate(&self.num_keys) as u32;
            if let Some(index) = self.indices.get(&track_number) {
                self.play_index(index.clone() as usize);
            } else {
                self.clear();
            }
        }
    }

    // Play the track selected from mouse input.
    pub fn play_mouse_selected(&mut self, selected: usize) {
        self.play_index(selected);
    }

    // Play the last track in the current playlist.
    pub fn play_last_track(&mut self) {
        self.play_index(self.last_index());
    }

    // Skip to next track in the playlist.
    pub fn next(&mut self) {
        self.clear();
        if self.index < self.last_index() {
            self.index += 1;
            self.file = self.playlist[self.index].clone();
            self.set_playback();
        } else {
            self.stop();
        }
    }

    // Skip to previous track in the playlist.
    pub fn previous(&mut self) {
        self.clear();
        if self.index > 0 {
            self.index -= 1;
            self.file = self.playlist[self.index].clone();
        }
        self.set_playback();
    }

    // Increase volume by 10%, to maximum of 120%.
    pub fn increase_volume(&mut self) -> u8 {
        if self.volume < 120 {
            self.volume += 10;
            if !self.is_muted {
                self.sink.set_volume(self.volume as f32 / 100.0);
            }
        }
        self.volume
    }

    // Decrease volume by 10%, to minimum of 0%.
    pub fn decrease_volume(&mut self) -> u8 {
        if self.volume > 0 {
            self.volume -= 10;
            if !self.is_muted {
                self.sink.set_volume(self.volume as f32 / 100.0);
            }
        }
        self.volume
    }

    // Toggles `is_muted` and sets the volume to reflect
    // this change. Returns the updated `is_muted`.
    pub fn toggle_mute(&mut self) -> bool {
        self.is_muted ^= true;
        self.sink.set_volume(if self.is_muted {
            0.0
        } else {
            self.volume as f32 / 100.0
        });
        self.is_muted
    }

    // Toggles `is_randomized` and removes the current next
    // track from the sink when `is_randomized` is true.
    pub fn toggle_randomization(&mut self) -> bool {
        self.next_track_queued = false;
        self.is_randomized ^= true;
        if self.is_randomized {
            self.sink.pop();
        }
        self.is_randomized
    }

    // Tries to get the path of a random player and a random index for that player.
    pub fn randomized(paths: &Vec<PathBuf>) -> Option<(PathBuf, usize)> {
        if paths.len() == 0 {
            return None;
        }
        let mut count = 0;
        while count < 10 {
            let target = utils::random(0..paths.len());
            let path = paths[target].to_owned();
            if let Ok((playlist, _)) = playlist(&path, false) {
                let index = utils::random(0..playlist.len());
                return Some((path, index));
            } else {
                count += 1;
                continue;
            }
        }
        None
    }

    // Sets the track to the previous, randomly selected, track.
    pub fn previous_random(&mut self) {
        if self.playlist.len() > 1 {
            let current = self.index;
            self.index = self.previous;
            self.previous = current;
            self.file = self.playlist[self.index].to_owned();
            self.next_track_queued = false;
            self.set_playback();
        }
    }

    // Sets the current track in a playlist randomly.
    pub fn next_random(&mut self) {
        if self.playlist.len() > 1 {
            let mut index = utils::random(0..self.playlist.len());
            if index == self.index {
                // A second chance to find a new index.
                index = utils::random(0..self.playlist.len());
            }
            self.previous = self.index;
            self.index = index;
            self.file = self.playlist[index].to_owned();
            self.next_track_queued = false;
            self.set_playback();
        }
    }

    // Seeks the playback to the input time in seconds.
    pub fn seek_to_sec(&mut self) {
        if !self.num_keys.is_empty() {
            let secs = utils::concatenate(&self.num_keys) as u64;
            let seek_time = Duration::new(secs, 0);
            self.seek_to_time(seek_time)
        }
    }

    // Seeks the playback to the input time in minutes.
    pub fn seek_to_min(&mut self) {
        if !self.num_keys.is_empty() {
            let secs = utils::concatenate(&self.num_keys) as u64;
            let seek_time = Duration::new(secs * 60, 0);
            self.seek_to_time(seek_time)
        }
    }

    // Increments the playback position by SEEK_TIME.
    pub fn step_forward(&mut self) {
        let elapsed = self.elapsed();
        self.seek_forward(SEEK_TIME, elapsed);
    }

    // Decrements the playback position by SEEK_TIME.
    pub fn step_backward(&mut self) {
        let elapsed = self.elapsed();
        self.seek_backward(SEEK_TIME, elapsed);
    }

    // Seeks the playback to the provided seek_time, in seconds.
    #[inline]
    pub fn seek_to_time(&mut self, seek_time: Duration) {
        let elapsed = self.elapsed();
        if seek_time < elapsed {
            let diff = elapsed - seek_time;
            self.seek_backward(diff, elapsed);
        } else {
            let diff = seek_time - elapsed;
            self.seek_forward(diff, elapsed);
        }
        self.num_keys.clear();
    }

    // Performs the seek operation in the forward direction.
    #[inline]
    fn seek_forward(&mut self, time: Duration, elapsed: Duration) {
        if !self.is_playing() {
            self.play_or_pause();
        }
        let duration = Duration::new(self.file.duration as u64, 0);
        if duration - elapsed < time + Duration::new(0, 500) {
            self.next()
        } else {
            let future = elapsed + time;
            if let Ok(_) = self.sink.try_seek(future) {
                self.last_started -= time;
            }
        }
    }

    // Performs the seek operation in the backward direction.
    #[inline]
    fn seek_backward(&mut self, time: Duration, elapsed: Duration) {
        if !self.is_playing() {
            self.play_or_pause();
        }
        if elapsed < time + Duration::new(0, 500) {
            self.stop();
            self.play();
        } else {
            let past = elapsed - time;
            if let Ok(_) = self.sink.try_seek(past) {
                if self.last_elapsed == Duration::ZERO {
                    self.last_started += time;
                } else if self.last_elapsed >= time {
                    self.last_elapsed -= time;
                } else {
                    let diff = time - self.last_elapsed;
                    self.last_elapsed = Duration::ZERO;
                    self.last_started += diff;
                }
            }
        }
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

    // Performs the function of a mixer by polling the player
    // sink during the layout phase of PlayerView.
    //
    // If playback is not randomized and there is a succeeding
    // track in the playlist, the next track is queued before the
    // current track completes. This is to ensure gapless playback.
    //
    // If playback is randomized, the next track is queued when
    // the current track completes.
    //
    // Finally, playback is stopped when the sink is emptied.
    //
    // Return values are for the automated player, where:
    // 0 => the player has completed.
    // 1 => the player has changed.
    // 2 => the player is unchanged.
    #[inline]
    pub fn poll(&mut self) -> usize {
        if !self.is_playing() {
            return 0;
        }
        if self.is_randomized {
            if self.sink.empty() {
                self.next_track_queued = true;
            }
        } else if self.sink.len() == 1 {
            if self.next_track_queued {
                self.last_started = Instant::now();
                self.last_elapsed = Duration::ZERO;
                self.index += 1;
                self.file = self.playlist[self.index].clone();
                self.next_track_queued = false;
                return 1;
            } else if self.index < self.playlist.len() - 1 {
                let file = self.playlist[self.index + 1].clone();
                let path = &file.path;
                if let Ok(source) = decode(path) {
                    self.sink.append(source);
                    self.next_track_queued = true;
                } else {
                    self.next();
                }
            }
        } else if self.sink.empty() {
            self.stop();
        }
        2
    }

    // Stdout for the automated player.
    pub fn stdout(&self) -> (String, usize) {
        let line = format!(
            "[tap player]: '{}' by '{}' ({}/{}) ",
            self.file.title,
            self.file.artist,
            self.index + 1,
            self.playlist.len()
        );
        let length = line.len();
        (line, length)
    }

    // Whether the player is playing or not.
    fn is_playing(&self) -> bool {
        self.status == PlayerStatus::Playing
    }

    // The index of the last track in the playlist.
    fn last_index(&self) -> usize {
        self.playlist.len() - 1
    }

    // Removes the stored keyboard inputs.
    fn clear(&mut self) {
        self.next_track_queued = false;
        self.num_keys.clear();
        self.timer_bool.set_false();
    }

    // Play the track at `index` in playlist.
    fn play_index(&mut self, index: usize) {
        self.stop();
        self.index = index;
        self.file = self.playlist[index].clone();
        self.clear();
        self.play();
    }

    // Convenience method to maintain `status` in new player instances.
    fn set_playback(&mut self) {
        self.sink.stop();
        self.last_elapsed = Duration::ZERO;

        if self.status != PlayerStatus::Stopped {
            if let Ok(source) = decode(&self.file.path) {
                self.sink.append(source);
                self.last_started = Instant::now();
            }
            if self.status == PlayerStatus::Paused {
                self.sink.pause()
            }
        }
    }

    // Apply volume setting to the audio sink.
    fn set_volume(&mut self) {
        if self.is_muted {
            self.sink.set_volume(0.0)
        } else {
            self.sink.set_volume(self.volume as f32 / 100.0);
        }
    }
}

// Returns the playlist and required size for the player on success.
pub fn playlist(
    path: &PathBuf,
    recurse: bool,
) -> Result<(Vec<AudioFile>, XY<usize>), anyhow::Error> {
    // The list of files to use in the player.
    let mut list: Vec<AudioFile> = vec![];
    // A value used to set an appropriate width for the player view.
    let mut width = 0;
    // The error we get if we can't create an audio file.
    let mut error: Option<anyhow::Error> = None;
    // The first child directory to recurse into.
    let mut next_path: Option<PathBuf> = None;

    // Build the playlist.
    if let Ok(iter) = path.read_dir() {
        for entry in iter {
            if let Ok(entry) = entry {
                let path = entry.path();
                if recurse && path.is_dir() && next_path.is_none() {
                    next_path = Some(path);
                } else {
                    update_playlist(&mut list, &mut width, &mut error, path)
                }
            }
        }
    } else {
        // If `path` is a file, create a playlist containing it.
        update_playlist(&mut list, &mut width, &mut error, path.clone())
    }

    //     for path in iter.filter_map(|e| e.ok()).map(|e| e.path()) {

    //         if recurse && path.is_dir() {
    //             return playlist(&path, recurse);
    //         } else {
    //             update_playlist(&mut list, &mut width, &mut error, path)
    //         }
    //     }
    // } else {
    //     // If `path` is a file, create a playlist containing it.
    //     update_playlist(&mut list, &mut width, &mut error, path.clone())
    // }

    if let Some(next_path) = next_path {
        if list.is_empty() {
            return playlist(&next_path, recurse);
        }
    }

    if let Some(first) = list.first() {
        width = max(width, first.album.len() + first.artist.len() + 1);
        _ = decode(&first.path)?;
    } else {
        // Use the correct path in error messages.
        let path = match recurse {
            true => args::search_root(),
            false => path.to_owned(),
        };
        // Handle errors.
        if list.is_empty() {
            bail!("'{}' is empty", path.display())
        } else {
            match error {
                Some(e) => bail!(e),
                None => bail!("no audio files detected in '{}'", path.display()),
            }
        }
    }

    list.sort();

    let size = XY {
        x: max(width + 19, 53),
        y: min(45, list.len() + 3),
    };

    Ok((list, size))
}

#[inline]
fn update_playlist(
    list: &mut Vec<AudioFile>,
    width: &mut usize,
    error: &mut Option<anyhow::Error>,
    path: PathBuf,
) {
    if valid_audio_ext(&path) {
        match AudioFile::new(path) {
            // Grow the `playlist` and update `width`.
            Ok(f) => {
                *width = max(*width, f.title.len());
                list.push(f)
            }
            // Save the first error encountered for error handling
            // in the event of an empty playlist.
            Err(e) => {
                if error.is_none() {
                    *error = Some(e)
                }
            }
        }
    }
}

pub fn decode(path: &PathBuf) -> Result<Decoder<BufReader<File>>, anyhow::Error> {
    let source = match File::open(path.as_path()) {
        Ok(inner) => match Decoder::new(BufReader::new(inner)) {
            Ok(s) => s,
            Err(_) => bail!("could not decode '{}'", path.display()),
        },
        Err(_) => bail!("could not open '{}'", path.display()),
    };
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{create_working_dir, find_assets_dir};

    #[test]
    fn test_playlist_recurse_success() {
        let root = create_working_dir(
            &["one", "one/two"],
            &[("one/two/foo.mp3", "test_mp3_audio.mp3")],
            &[],
        )
        .expect("create temp dir")
        .into_path();

        let res = playlist(&root, true);
        assert!(
            res.expect("should be ok").0.len() == 1,
            "Expected to find the audio file in leaf directory"
        );
    }

    #[test]
    fn test_playlist_recurse_error() {
        let root = create_working_dir(
            &["one", "one/two"],
            &[("one/two/foo.mp3", "test_mp3_audio.mp3")],
            &[],
        )
        .expect("create temp dir")
        .into_path();

        let res = playlist(&root, false);
        assert!(
            res.is_err(),
            "Expected to not find the audio file in leaf directory"
        );
    }

    #[test]
    fn test_playlist_mp3_success() {
        let root = find_assets_dir().join("test_mp3_audio.mp3");
        let (playlist, _) = playlist(&root, false).expect("should create a valid playlist");

        assert_eq!(playlist[0].title, "test_audio_mp3");
    }

    #[test]
    fn test_playlist_flac_success() {
        let root = find_assets_dir().join("test_flac_audio.flac");
        let (playlist, _) = playlist(&root, false).expect("should create a valid playlist");

        assert_eq!(playlist[0].title, "test_audio_flac");
    }

    #[test]
    fn test_playlist_m4a_success() {
        let root = find_assets_dir().join("test_m4a_audio.m4a");
        let (playlist, _) = playlist(&root, false).expect("should create a valid playlist");

        assert_eq!(playlist[0].title, "test_audio_m4a");
    }

    #[test]
    fn test_playlist_wav_success() {
        let root = find_assets_dir().join("test_wav_audio.wav");
        let (playlist, _) = playlist(&root, false).expect("should create a valid playlist");

        assert_eq!(playlist[0].title, "test_audio_wav");
    }

    #[test]
    fn test_playlist_ogg_success() {
        let root = find_assets_dir().join("test_ogg_audio.ogg");
        let (playlist, _) = playlist(&root, false).expect("should create a valid playlist");

        assert_eq!(playlist[0].title, "test_audio_ogg");
    }

    #[test]
    fn test_playlist_assets_length() {
        let root = find_assets_dir();
        let (playlist, _) = playlist(&root, false).expect("should create a valid playlist");

        assert_eq!(
            playlist.len(),
            5,
            "\n\n\
            {:?} contains 5 test data and 3 error injection data. \
            The playlist should only include the test data.\n",
            root
        );
    }

    #[test]
    fn test_playlist_assets_size() {
        let root = find_assets_dir();
        let (_, size) = playlist(&root, false).expect("should create a valid playlist");

        assert_eq!((size.x, size.y), (53, 8));
    }

    #[test]
    fn test_playlist_empty_error() {
        let root = create_working_dir(&["one"], &[], &[])
            .expect("create temp dir")
            .into_path();

        let res = playlist(&root, false);
        assert!(
            res.is_err(),
            "Providing the path to an empty directory should yield an error"
        );
    }
}
