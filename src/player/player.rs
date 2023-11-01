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
use crate::utils::{concatenate, random};

use super::{is_valid, AudioFile, PlayerOpts, PlayerStatus, StatusToBytes};

pub type PlayerResult = Result<(Player, bool, XY<usize>), anyhow::Error>;

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
    pub is_queued: bool,
    // Whether the player is playing, paused or stopped.
    pub status: PlayerStatus,
    // The list of numbers from last keyboard input,
    pub number_keys: Vec<usize>,
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
            last_elapsed: Duration::default(),
            previous: 0,
            number_keys: vec![],
            is_queued: false,
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
        let path = &self.file.path;

        if let Ok(source) = get_source(path) {
            self.sink.append(source);
            self.sink.play();
            self.status = PlayerStatus::Playing;
            self.last_started = Instant::now();
        } else {
            self.next()
        }
    }

    pub fn play_or_pause(&mut self) -> u8 {
        self.clear();

        match self.status {
            PlayerStatus::Paused => self.resume(),
            PlayerStatus::Playing => self.pause(),
            PlayerStatus::Stopped => self.play(),
        };

        self.status.to_u8()
    }

    pub fn stop(&mut self) -> u8 {
        self.clear();

        match self.status {
            PlayerStatus::Stopped => {}
            _ => {
                self.sink.stop();
                self.status = PlayerStatus::Stopped;
                self.last_elapsed = Duration::default()
            }
        }

        self.status.to_u8()
    }

    pub fn play_selection(&mut self) {
        if self.select_track() {
            self.play_or_pause();
        }
    }

    pub fn play_last_track(&mut self) {
        if self.select_last_track() {
            self.play_or_pause();
        }
    }

    // Removes the stored keyboard inputs.
    fn clear(&mut self) {
        self.is_queued = false;
        self.number_keys.clear();
        self.timer_bool.set_false();
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
        if self.timer_bool.is_true() {
            self.select_first_track()
        } else {
            self.timer_bool.set();
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

        self.set_playback();
    }

    pub fn previous(&mut self) {
        self.clear();

        if self.index > 0 {
            self.index -= 1;
            self.file = self.playlist[self.index].clone();
        }

        self.set_playback();
    }

    // Convenience method to maintain `status` in new player instances.
    pub fn set_playback(&mut self) {
        match self.status {
            PlayerStatus::Paused => {
                self.stop();
                self.play();
                self.pause();
            }
            PlayerStatus::Playing => {
                self.stop();
                self.play();
            }
            PlayerStatus::Stopped => {
                self.stop();
                self.play();
                self.stop();
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

    // Return values are for the automated player, where:
    // 0 => the player has completed.
    // 1 => the player has changed.
    // 2 => the player is unchanged.
    pub fn poll(&mut self) -> usize {
        if !self.is_playing() {
            return 0;
        }
        if self.is_randomized {
            if self.sink.empty() {
                self.is_queued = true;
            }
        } else if self.sink.len() == 1 {
            if self.is_queued {
                self.last_started = Instant::now();
                self.index += 1;
                self.file = self.playlist[self.index].clone();
                self.is_queued = false;
                return 1;
            } else if self.index < self.playlist.len() - 1 {
                let file = self.playlist[self.index + 1].clone();
                let path = &file.path;

                if let Ok(source) = get_source(path) {
                    self.sink.append(source);
                    self.is_queued = true;
                } else {
                    self.next();
                }
            }
        } else if self.sink.empty() {
            self.stop();
        }
        2
    }

    // Tries to get the path of a random player and a random index for that player.
    pub fn randomized(paths: &Vec<PathBuf>) -> Option<(PathBuf, usize)> {
        if paths.len() == 0 {
            return None;
        }

        let mut count = 0;
        loop {
            if count > 10 {
                // Give up after a while.
                return None;
            }
            let target = random(0..paths.len());
            let path = paths[target].to_owned();
            if let Ok((playlist, _)) = playlist(&path, false) {
                let index = random(0..playlist.len());
                return Some((path, index));
            } else {
                count += 1;
                continue;
            }
        }
    }

    // Sets the current track in a playlist randomly.
    pub fn next_random(&mut self) {
        if self.playlist.len() > 1 {
            let mut index = random(0..self.playlist.len());
            if index == self.index {
                // A second chance to find a new index.
                index = random(0..self.playlist.len());
            }
            self.previous = self.index;
            self.index = index;
            self.file = self.playlist[index].to_owned();
            self.is_queued = false;
            self.set_playback();
        }
    }

    // Sets the track to the previous, randomly selected, track.
    pub fn previous_random(&mut self) {
        if self.playlist.len() > 1 {
            let current = self.index;
            self.index = self.previous;
            self.previous = current;
            self.file = self.playlist[self.index].to_owned();
            self.is_queued = false;
            self.set_playback();
        }
    }

    pub fn set_volume(&mut self) {
        if self.is_muted {
            self.sink.set_volume(0.0)
        } else {
            self.sink.set_volume(self.volume as f32 / 100.0);
        }
    }

    pub fn increase_volume(&mut self) -> u8 {
        if self.volume < 120 {
            self.volume += 10;
            if !self.is_muted {
                self.sink.set_volume(self.volume as f32 / 100.0);
            }
        }

        self.volume
    }

    pub fn decrease_volume(&mut self) -> u8 {
        if self.volume > 0 {
            self.volume -= 10;
            if !self.is_muted {
                self.sink.set_volume(self.volume as f32 / 100.0);
            }
        }

        self.volume
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.is_muted ^= true;

        match self.is_muted {
            true => self.sink.set_volume(0.0),
            false => self.sink.set_volume(self.volume as f32 / 100.0),
        };

        self.is_muted
    }

    pub fn toggle_randomization(&mut self) {
        self.is_queued = false;
        if !self.is_randomized {
            self.sink.pop();
        }
        self.is_randomized ^= true;
    }

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
}

// Returns the playlist and required size for the player on success.
pub fn playlist(
    path: &PathBuf,
    recurse: bool,
) -> Result<(Vec<AudioFile>, XY<usize>), anyhow::Error> {
    // The list of files to use in the player.
    let mut audio_files = vec![];
    // An intermediate value used in calculating the player width.
    let mut width = 0;
    // True when `path` is an empty directory.
    let mut is_empty = true;
    // The error we get if we can't create an audio file.
    let mut error: Option<anyhow::Error> = None;

    if path.is_dir() {
        for entry in path.read_dir()? {
            is_empty = false;
            if let Ok(entry) = entry {
                let path = entry.path();
                if recurse && path.is_dir() {
                    return playlist(&path, recurse);
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
        None => {
            let path = match recurse {
                true => args::search_root(),
                false => path.to_owned(),
            };

            // Give an appropriate error if we fail to find any valid files.
            match is_empty {
                true => bail!("'{}' is empty", path.display()),
                false => match error {
                    Some(e) => bail!(e),
                    None => bail!("no audio files detected in '{}'", path.display()),
                },
            }
        }
    }

    audio_files.sort();

    let size = XY {
        x: max(width + 19, 53),
        y: min(45, audio_files.len() + 3),
    };

    Ok((audio_files, size))
}

// Returns `Ok` if the file can be decoded.
fn can_decode(audio_file: &AudioFile) -> Result<(), anyhow::Error> {
    let _ = get_source(&audio_file.path)?;
    Ok(())
}

fn get_source(path: &PathBuf) -> Result<Decoder<BufReader<File>>, anyhow::Error> {
    let source = match File::open(path.as_path()) {
        Ok(inner) => match Decoder::new(BufReader::new(inner)) {
            Ok(s) => s,
            Err(_) => bail!("could not decode '{}", path.display()),
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
