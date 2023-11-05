use core::cmp::Ordering;
use std::{collections::HashSet, path::PathBuf};

use anyhow::bail;
use lofty::{Accessor, AudioFile as LoftyAudioFile, Probe, TaggedFileExt};

// The set of valid audio file extensions.
lazy_static::lazy_static! {
    pub static ref AUDIO_FORMATS: HashSet<&'static str> = create_set();
}

#[derive(Clone, Debug, Eq, PartialEq, Ord)]
pub struct AudioFile {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: Option<u32>,
    pub track: u32,
    pub duration: usize,
}

impl AudioFile {
    pub fn new(path: PathBuf) -> Result<Self, anyhow::Error> {
        let file = match Probe::open(&path) {
            Ok(f) => f,
            Err(e) => bail!("could not probe '{}'\n-`{}`", path.display(), e),
        };

        let tagged_file = match file.read() {
            Ok(f) => f,
            Err(e) => bail!("failed to read '{}'\n- `{}`", path.display(), e),
        };

        let tag = match tagged_file.primary_tag() {
            Some(primary_tag) => primary_tag,
            None => match tagged_file.first_tag().ok_or(()) {
                Ok(t) => t,
                Err(_) => bail!("no tags found for '{}'", path.display()),
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
            artist,
            path,
            duration,
        };

        Ok(audio_file)
    }
}

// Order by Album -> Track / Title
impl PartialOrd for AudioFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.album
                .cmp(&other.album)
                .then(match self.track == other.track {
                    true => self.title.cmp(&other.title),
                    false => self.track.cmp(&other.track),
                }),
        )
    }
}

// Returns true if the file extension is a valid format.
pub fn valid_audio_ext(p: &PathBuf) -> bool {
    let ext = p.extension().unwrap_or_default().to_str().unwrap();
    AUDIO_FORMATS.contains(&ext)
}

fn create_set() -> HashSet<&'static str> {
    let mut m = HashSet::new();
    m.insert("aac");
    m.insert("flac");
    m.insert("mp3");
    m.insert("m4a");
    m.insert("ogg");
    m.insert("wav");
    m.insert("wma");
    m
}
