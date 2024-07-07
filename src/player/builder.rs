use std::path::PathBuf;

use anyhow::bail;
use cursive::Cursive;

use crate::session_data::SessionData;
use crate::utils::{self, InnerType};

use super::{
    player::{playlist, PlayerResult},
    Player, PlayerOpts,
};

#[derive(PartialEq)]
pub enum PlayerBuilder {
    FuzzyFinder,
    PreviousAlbum,
    PreviousTrack,
    RandomAlbum,
    RandomTrack,
}

impl PlayerBuilder {
    pub fn from(&self, path: Option<PathBuf>, siv: &mut Cursive) -> PlayerResult {
        match self {
            Self::FuzzyFinder => Self::fuzzy(path, siv),
            Self::PreviousAlbum | Self::PreviousTrack => Self::previous(self, siv),
            Self::RandomAlbum | Self::RandomTrack => Self::random(self, siv),
        }
    }

    pub fn create(path: PathBuf) -> PlayerResult {
        let opts = PlayerOpts::default();
        Player::new(path, 0, opts, false)
    }

    fn previous(&self, siv: &mut Cursive) -> PlayerResult {
        let ((path, mut index), opts) = siv
            .with_user_data(|(opts, _, queue): &mut InnerType<SessionData>| {
                let (path, index) = queue.front().expect("should always exist").to_owned();
                let opts: PlayerOpts = (*opts).into();

                if queue.len() != 1 {
                    queue.swap(0, 1);
                    ((Some(path), index), opts)
                } else {
                    ((None, index), opts)
                }
            })
            .expect("should be set on init");

        if Self::PreviousAlbum.eq(self) {
            index = 0
        }

        let is_randomized = Self::PreviousTrack.eq(self);

        match path {
            Some(path) => Player::new(path, index, opts, is_randomized),
            None => bail!("path not set"),
        }
    }

    fn random(&self, siv: &mut Cursive) -> PlayerResult {
        let ((path, mut index), opts) = siv
            .with_user_data(|(opts, paths, queue): &mut InnerType<SessionData>| {
                let opts: PlayerOpts = (*opts).into();
                let (path, index) = queue.back().expect("should always exist").to_owned();

                if queue.len() == 1 {
                    let front = queue.front().expect("should always exist").to_owned();
                    queue.push_back(front);
                } else {
                    queue.pop_front();
                }

                let next_random = match Player::randomized(paths) {
                    Some(track) => track,
                    None => {
                        let path = path.to_owned();
                        let upper_bound = playlist(&path).expect("should always exist").0.len();
                        let index = utils::random(0..upper_bound);
                        (path, index)
                    }
                };

                queue.push_back(next_random);

                ((path, index), opts)
            })
            .expect("should be set on init");

        if Self::RandomAlbum.eq(self) {
            index = 0;
        }

        Player::new(path, index, opts, Self::RandomTrack.eq(self))
    }

    fn fuzzy(path: Option<PathBuf>, siv: &mut Cursive) -> PlayerResult {
        let path = path.expect("path should be provided by fuzzy-finder");

        let opts = siv
            .with_user_data(|(opts, _, queue): &mut InnerType<SessionData>| {
                let opts: PlayerOpts = (*opts).into();

                if queue.len() == 1 {
                    queue.push_front((path.clone(), 0));
                    queue.push_front((path.clone(), 0));
                } else {
                    queue.pop_front();
                    queue.insert(1, (path.clone(), 0));
                }

                opts
            })
            .expect("should be set on init");

        Player::new(path, 0, opts, false)
    }
}
