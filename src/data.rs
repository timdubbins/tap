use std::collections::VecDeque;
use std::path::PathBuf;

use anyhow::bail;

use crate::fuzzy::{leaf_paths, FuzzyItem};
use crate::player::{Player, PlayerOpts};
use crate::utils::IntoInner;

pub struct UserData {
    opts: PlayerOpts,
    paths: Vec<PathBuf>,
    queue: VecDeque<(PathBuf, usize)>,
}

impl UserData {
    pub fn new(path: &PathBuf, items: &Vec<FuzzyItem>) -> Result<Self, anyhow::Error> {
        let paths = leaf_paths(&items);
        let queue: VecDeque<(PathBuf, usize)> = match Player::randomized(&paths) {
            Some(first) => VecDeque::from([first]),
            None => bail!("could not find audio files in '{}'", path.display()),
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
