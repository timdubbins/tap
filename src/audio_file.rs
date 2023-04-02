use core::cmp::Ordering;
use std::path::PathBuf;

use lofty::{Accessor, AudioFile as LoftyAudioFile, Probe, TaggedFileExt};

#[derive(Clone, Debug, Eq, PartialEq, Ord)]
pub struct AudioFile {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: Option<u32>,
    pub track: u32,
    duration: u64,
    pub duration_display: String,
}

impl AudioFile {
    pub fn new(path: PathBuf) -> Self {
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
            title: tag.title().as_deref().unwrap_or("None").trim().to_string(),
            artist: tag.artist().as_deref().unwrap_or("None").trim().to_string(),
            album: tag.album().as_deref().unwrap_or("None").trim().to_string(),
            year: tag.year(),
            track: tag.track().unwrap_or(0),
            duration_display: format!("{:02}:{:02}", duration / 60, duration % 60),
            path,
            duration,
        }
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