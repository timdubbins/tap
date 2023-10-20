use std::collections::VecDeque;
use std::path::PathBuf;

use anyhow::bail;

use crate::fuzzy::{leaf_paths, FuzzyItem};
use crate::player::{Player, PlayerOpts};
use crate::utils::IntoInner;

// The path and track number for an audio file.
type Track = (PathBuf, usize);

#[derive(Debug)]
pub struct UserData {
    opts: PlayerOpts,
    // The list of paths from Vec<FuzzyItem>.
    paths: Vec<PathBuf>,
    // The queue of `track`s that takes one of two forms:
    // [`current_track`] or [`previous_track`, `current_track`, `next_random_track`]
    queue: VecDeque<Track>,
}

impl UserData {
    pub fn new(path: &PathBuf, items: &Vec<FuzzyItem>) -> Result<Self, anyhow::Error> {
        let paths = leaf_paths(&items);
        let queue: VecDeque<Track> = match Player::randomized(&paths) {
            Some(first) => VecDeque::from([first]),
            None => bail!("no audio files detected in '{}'", path.display()),
        };

        let data = Self {
            opts: PlayerOpts::default(),
            paths,
            queue,
        };

        Ok(data)
    }
}

impl IntoInner for UserData {
    type T = (
        (u8, u8, bool, bool),
        Vec<PathBuf>,
        VecDeque<(PathBuf, usize)>,
    );

    fn into_inner(self) -> Self::T {
        (self.opts.into_inner(), self.paths, self.queue)
    }
}

impl Into<UserData>
    for (
        (u8, u8, bool, bool),
        Vec<PathBuf>,
        VecDeque<(PathBuf, usize)>,
    )
{
    fn into(self) -> UserData {
        UserData {
            opts: self.0.into(),
            paths: self.1,
            queue: self.2,
        }
    }
}
