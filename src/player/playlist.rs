use std::{cmp::max, fs, path::PathBuf};

use rand::seq::IteratorRandom;

use {
    anyhow::{anyhow, bail},
    cursive::XY,
    rand::{seq::SliceRandom, thread_rng},
};

use crate::{finder::FuzzyDir, player::AudioFile, TapError};

const MIN_WIDTH: usize = 30;
const X_PADDING: usize = 15;
const Y_PADDING: usize = 3;
const ROW_PADDING: usize = 6;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Playlist {
    pub fdir: FuzzyDir,
    pub audio_files: Vec<AudioFile>,
    pub index: usize,
    pub xy_size: XY<usize>,
}

impl Playlist {
    fn new(fdir: FuzzyDir, audio_files: Vec<AudioFile>, width: usize) -> Self {
        let xy_size = XY {
            x: max(width, MIN_WIDTH) + X_PADDING,
            y: audio_files.len() + Y_PADDING,
        };

        Playlist {
            fdir,
            audio_files,
            xy_size,
            index: 0,
        }
    }

    pub fn process(path: &PathBuf, bail_on_subdir: bool) -> Result<Self, TapError> {
        if path.is_file() {
            Playlist::process_file(path)
        } else if path.is_dir() {
            Playlist::process_dir(path, bail_on_subdir)
        } else {
            bail!("{:?} is not a file or directory", path);
        }
    }

    fn process_file(p: &PathBuf) -> Result<Self, TapError> {
        if !AudioFile::validate_format(p) {
            bail!("{:?} is not a valid audio file", p);
        }
        let af = AudioFile::new(p.clone())?;
        af.decode()?;
        let width = max(
            af.title.len() + ROW_PADDING,
            af.artist.len() + af.album.len(),
        );
        let fdir = FuzzyDir::default();

        Ok(Self::new(fdir, vec![af], width))
    }

    fn process_dir(p: &PathBuf, bail_on_subdir: bool) -> Result<Self, TapError> {
        let mut width = 0;
        let mut audio_files = Vec::new();
        for entry in fs::read_dir(p).map_err(|e| anyhow!("failed to read {:?}: {}", p, e))? {
            let entry = entry?;
            let path = entry.path();
            if bail_on_subdir && path.is_dir() {
                bail!("Directory {:?} contains subdirectories", p);
            }
            if AudioFile::validate_format(&path) {
                if let Ok(af) = AudioFile::new(path.clone()) {
                    let len = max(
                        af.title.len() + ROW_PADDING,
                        af.artist.len() + af.album.len(),
                    );
                    width = width.max(len);
                    audio_files.push(af);
                }
            }
        }
        if audio_files.is_empty() {
            bail!("No audio files detected in {:?}", p);
        }

        audio_files.first().unwrap().decode()?;
        audio_files.sort();
        let fdir = FuzzyDir::default();

        Ok(Self::new(fdir, audio_files, width))
    }

    pub fn set_random_index(&mut self) {
        let len = self.audio_files.len();
        let mut rng = rand::thread_rng();

        if len >= 2 {
            self.index = (0..len)
                .filter(|&i| i == 0 || i != self.index)
                .choose(&mut rng)
                .unwrap_or(0);
        } else {
            self.index = 0;
        }
    }

    pub fn some_randomized(fdirs: &[FuzzyDir]) -> Option<Playlist> {
        let mut rng = thread_rng();

        (0..10).find_map(|_| {
            fdirs
                .choose(&mut rng)
                .and_then(|dir| Playlist::try_from(dir.clone()).ok())
        })
    }

    pub fn randomized_track(current: Playlist, fdirs: &[FuzzyDir]) -> Playlist {
        let mut rng = thread_rng();

        let mut playlist = (0..10)
            .find_map(|_| {
                fdirs
                    .choose(&mut rng)
                    .and_then(|fdir| Playlist::try_from(fdir.clone()).ok())
                    .filter(|next| *next != current)
            })
            .unwrap_or(current);

        playlist.set_random_index();

        playlist
    }

    pub fn randomized(current: Playlist, fdirs: &[FuzzyDir]) -> Playlist {
        let mut rng = thread_rng();

        (0..10)
            .find_map(|_| {
                fdirs
                    .choose(&mut rng)
                    .and_then(|fdir| Playlist::try_from(fdir.clone()).ok())
                    .filter(|next| *next != current)
            })
            .unwrap_or(current)
    }

    pub fn get_next_track(&self) -> Option<AudioFile> {
        self.audio_files
            .get(self.index + 1)
            .map(|file| file.clone())
    }

    pub fn is_last_track(&self) -> bool {
        self.index == self.audio_files.len() - 1
    }
}

impl TryFrom<FuzzyDir> for Playlist {
    type Error = TapError;
    fn try_from(fdir: FuzzyDir) -> Result<Self, Self::Error> {
        let mut playlist = Playlist::process_dir(&fdir.path, false)?;
        playlist.fdir = fdir;

        Ok(playlist)
    }
}
