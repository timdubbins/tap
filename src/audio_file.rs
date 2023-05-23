use core::cmp::Ordering;
use std::path::PathBuf;

use anyhow::bail;
use lofty::{Accessor, AudioFile as LoftyAudioFile, Probe, TaggedFileExt};

#[derive(Clone, Debug, Eq, PartialEq, Ord)]
pub struct AudioFile {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: Option<u32>,
    pub track: u32,
    pub duration: usize,
    pub offset: usize,
}

impl AudioFile {
    pub fn new(path: PathBuf) -> Result<Self, anyhow::Error> {
        let file = Probe::open(&path)
            .expect("the path of `tagged_file` is provided by `Player::create_playlist()`");

        let tagged_file = match file.read() {
            Ok(f) => f,
            Err(e) => bail!("Failed to read '{:?}'. Error: {}", path, e),
        };

        let tag = match tagged_file.primary_tag() {
            Some(primary_tag) => primary_tag,
            None => match tagged_file.first_tag().ok_or(()) {
                Ok(t) => t,
                Err(_) => bail!("No tags found for '{:?}", path),
            },
        };

        let properties = tagged_file.properties();
        let artist = tag.artist().as_deref().unwrap_or("None").trim().to_string();
        let duration = properties.duration().as_secs() as usize;

        let audio_file = Self {
            album: tag.album().as_deref().unwrap_or("None").trim().to_string(),
            title: tag.title().as_deref().unwrap_or("None").trim().to_string(),
            year: tag.year(),
            track: tag.track().unwrap_or(0),
            offset: 4 + artist.chars().count(),
            artist,
            path,
            duration,
        };

        Ok(audio_file)
    }
}

// Order by Artist -> Album -> Track -> Title
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
