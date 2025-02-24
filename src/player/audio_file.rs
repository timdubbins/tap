use std::{cmp::Ordering, collections::HashSet, fs::File, io::BufReader, path::PathBuf};

use {
    anyhow::{anyhow, bail},
    lofty::{
        prelude::{Accessor, AudioFile as LoftyAudioFile, TaggedFileExt},
        probe::Probe,
    },
    once_cell::sync::Lazy,
    rodio::Decoder,
};

use crate::TapError;

pub static AUDIO_FORMATS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut m = HashSet::new();
    m.insert("aac");
    m.insert("flac");
    m.insert("mp3");
    m.insert("m4a");
    m.insert("ogg");
    m.insert("wav");
    m.insert("wma");
    m
});

// A struct representing metadata for an audio file.
#[derive(Clone, Debug, Eq, PartialEq, Ord)]
pub struct AudioFile {
    // The file path to the audio file.
    pub path: PathBuf,
    // The title of the audio track.
    pub title: String,
    // The artist that performed the track.
    pub artist: String,
    // The album the track belongs to.
    pub album: String,
    // The release year of the track, if available.
    pub year: Option<u32>,
    // The track number on the album.
    pub track: u32,
    // The duration of the track in seconds.
    pub duration: usize,
}

impl AudioFile {
    pub fn new(path: PathBuf) -> Result<Self, TapError> {
        let tagged_file = Probe::open(&path)
            .map_err(|e| anyhow!("faied to probe {:?}: {}", path, e))?
            .read()
            .map_err(|e| anyhow!("failed to read {:?}: {}", path, e))?;

        let tag = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
            .ok_or_else(|| anyhow!("no tags found for {:?}", path))?;

        let audio_file = Self {
            artist: tag.artist().as_deref().unwrap_or("None").trim().to_string(),
            album: tag.album().as_deref().unwrap_or("None").trim().to_string(),
            title: tag.title().as_deref().unwrap_or("None").trim().to_string(),
            year: tag.year(),
            track: tag.track().unwrap_or(0),
            duration: tagged_file.properties().duration().as_secs() as usize,
            path,
        };

        Ok(audio_file)
    }

    pub fn decode(&self) -> Result<Decoder<BufReader<File>>, TapError> {
        let source = match File::open(self.path.clone()) {
            Ok(inner) => match Decoder::new(BufReader::new(inner)) {
                Ok(s) => s,
                Err(_) => bail!("could not decode {:?}", self.path),
            },
            Err(_) => bail!("could not open {:?}", self.path),
        };
        Ok(source)
    }

    // Checks if the given file path has a valid audio file extension.
    pub fn validate_format(p: &PathBuf) -> bool {
        let ext = p.extension().unwrap_or_default().to_str().unwrap();
        AUDIO_FORMATS.contains(&ext)
    }
}

impl PartialOrd for AudioFile {
    // Sorts `AudioFile` instances by album, track number, and finally by title if needed.
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
