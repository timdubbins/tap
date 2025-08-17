use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use {
    anyhow::bail,
    bincode::{Decode, Encode},
    rayon::iter::{ParallelBridge, ParallelIterator},
    walkdir::WalkDir,
};

use crate::{config::Config, player::Playlist, FuzzyDir, TapError};

const BATCH_SIZE: usize = 200;

#[derive(Decode, Encode, Clone, Default)]
pub struct Library {
    pub root: PathBuf,
    pub fdirs: Vec<FuzzyDir>,
}

impl Library {
    pub fn new(root: &PathBuf, sequential: bool) -> Self {
        let entries = Self::walk_entries(root);

        let fdirs: Vec<FuzzyDir> = if sequential {
            entries
                .filter(FuzzyDir::is_visible_dir)
                .filter_map(|entry| FuzzyDir::new(entry).ok())
                .collect()
        } else {
            entries
                .par_bridge()
                .filter(FuzzyDir::is_visible_dir)
                .filter_map(|entry| FuzzyDir::new(entry).ok())
                .collect()
        };

        Self {
            fdirs,
            root: root.clone(),
        }
    }

    pub fn first(root: &PathBuf) -> Self {
        let first = Self::walk_entries(root)
            .filter(FuzzyDir::is_visible_dir)
            .filter_map(|entry| FuzzyDir::new(entry).ok())
            .filter(|fdir| fdir.contains_audio)
            .take(1);

        Self {
            fdirs: first.collect::<Vec<FuzzyDir>>(),
            root: root.clone(),
        }
    }

    pub fn load_in_background(config: Config, lib_tx: Sender<LibraryEvent>) {
        let (fdir_tx, fdir_rx) = mpsc::channel::<FuzzyDir>();
        let search_root: PathBuf;

        match Self::deserialize(&config) {
            Ok(library) => {
                search_root = library.root.clone();
                _ = lib_tx.send(LibraryEvent::Init(library));
                LibraryEvent::spawn_update_cache(&search_root, fdir_rx, lib_tx)
            }
            Err(_) if config.use_default_path && config.default_path.is_some() => {
                search_root = config.default_path.clone().unwrap();
                LibraryEvent::spawn_update_cache(&search_root, fdir_rx, lib_tx)
            }
            _ => {
                search_root = config.search_root.clone();
                LibraryEvent::spawn_batch(&search_root, fdir_rx, lib_tx);
            }
        }

        let entries = Self::walk_entries(&search_root);

        if config.sequential {
            entries.for_each(|entry| Self::try_send_fdir(entry, &fdir_tx));
        } else {
            entries
                .par_bridge()
                .for_each_with(fdir_tx, |tx, entry| Self::try_send_fdir(entry, tx));
        }
    }

    pub fn serialize(&self) -> Result<(), TapError> {
        let cfg = bincode::config::standard();
        let data = bincode::encode_to_vec(self, cfg)?;
        let path = cache_path()?;
        let mut file = File::create(path)?;
        file.write_all(&data)?;
        Ok(())
    }

    pub fn deserialize(config: &Config) -> Result<Self, TapError> {
        let cache_path = cache_path()?;
        let mut file = File::open(&cache_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if buffer.is_empty() {
            bail!("Cache not set")
        }

        let cfg = bincode::config::standard();
        let (root, bytes) = bincode::decode_from_slice::<PathBuf, _>(&buffer[..], cfg)?;

        if Some(&root) != config.default_path.as_ref() {
            bail!("Cache is dirty")
        }

        if !config.use_default_path && config.search_root != root {
            bail!("Cache not used");
        }

        let fdirs = bincode::decode_from_slice::<Vec<FuzzyDir>, _>(&buffer[bytes..], cfg)?.0;

        Ok(Self { root, fdirs })
    }

    pub fn audio_count(&self) -> usize {
        self.iter()
            .filter(|fdir| fdir.contains_audio)
            .take(2)
            .count()
    }

    pub fn audio_dirs(&self) -> Vec<FuzzyDir> {
        self.iter().filter(|fdir| fdir.contains_audio).collect()
    }

    // Finds the path of the first dir with audio.
    pub fn first_playlist(&self) -> Result<Playlist, TapError> {
        match self.iter().find(|fdir| fdir.contains_audio) {
            Some(dir) => Playlist::try_from(dir.clone()),
            None => Playlist::process(&self.root.clone(), false),
        }
    }

    pub fn parent_of(&self, child: Option<FuzzyDir>) -> Self {
        let fdirs = child
            .filter(|child| child.depth > 0)
            .map(|child| {
                self.fdirs
                    .iter()
                    .filter(|fdir| fdir.is_parent_of(&child) && fdir.path != self.root)
                    .collect()
            })
            .unwrap_or(self.fdirs.clone());

        let fdirs = if fdirs.is_empty() {
            self.fdirs.clone()
        } else {
            fdirs
        };

        Self {
            root: self.root.clone(),
            fdirs,
        }
    }

    pub fn apply_filter(&self, filter: &LibraryFilter) -> Library {
        use LibraryFilter::*;

        let predicate: Box<dyn Fn(&FuzzyDir) -> bool> = match filter {
            Artist => Box::new(|fdir| fdir.contains_subdir),
            Album => Box::new(|fdir| fdir.contains_audio),
            Depth(depth) => Box::new(|fdir| fdir.depth == *depth),
            Key(key) => Box::new(|fdir| !fdir.contains_audio && fdir.key == *key),
            _ => return self.clone(),
        };

        let fdirs = self
            .fdirs
            .iter()
            .filter(|dir| predicate(dir))
            .cloned()
            .collect();

        Self {
            fdirs,
            root: self.root.clone(),
        }
    }

    fn walk_entries(root: &PathBuf) -> impl Iterator<Item = walkdir::DirEntry> {
        WalkDir::new(root).into_iter().filter_map(Result::ok)
    }

    fn try_send_fdir(entry: walkdir::DirEntry, tx: &Sender<FuzzyDir>) {
        if FuzzyDir::is_visible_dir(&entry) {
            if let Ok(fdir) = FuzzyDir::new(entry) {
                _ = tx.send(fdir);
            }
        }
    }
}

#[derive(Debug)]
pub enum LibraryEvent {
    Batch(Vec<FuzzyDir>),
    Init(Library),
    Finished(Option<Library>),
}

impl LibraryEvent {
    fn spawn_batch(search_root: &PathBuf, fdir_rx: Receiver<FuzzyDir>, lib_tx: Sender<Self>) {
        let root = search_root.clone();

        thread::spawn(move || {
            let mut batch = Vec::with_capacity(BATCH_SIZE);
            let mut first_batch_sent = false;

            for fd in fdir_rx {
                batch.push(fd.clone());
                if batch.len() >= BATCH_SIZE {
                    if !first_batch_sent {
                        let library = Library {
                            root: root.clone(),
                            fdirs: batch.clone(),
                        };
                        _ = lib_tx.send(Self::Init(library));
                        first_batch_sent = true;
                    } else {
                        _ = lib_tx.send(Self::Batch(batch.clone()));
                    }

                    batch.clear();
                }
            }

            if !batch.is_empty() {
                if !first_batch_sent {
                    _ = lib_tx.send(Self::Init(Library { root, fdirs: batch }));
                } else {
                    _ = lib_tx.send(Self::Batch(batch));
                }
            }

            _ = lib_tx.send(Self::Finished(None));
        });
    }

    fn spawn_update_cache(
        search_root: &PathBuf,
        fdir_rx: Receiver<FuzzyDir>,
        lib_tx: Sender<Self>,
    ) {
        let root = search_root.clone();

        thread::spawn(move || {
            let mut full_library = Library {
                root: root.clone(),
                fdirs: Vec::new(),
            };

            for fdir in fdir_rx {
                full_library.fdirs.push(fdir);
            }

            _ = full_library.serialize();
            _ = lib_tx.send(Self::Finished(Some(full_library)));
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LibraryFilter {
    // All dirs
    Unfiltered,
    // Dir depth from search root (1-4)
    Depth(usize),
    // All artists
    Artist,
    // All ablums
    Album,
    // All artists starting with key (A-Z)
    Key(char),
    // The parent of current dir
    Parent(Option<FuzzyDir>),
}

impl Deref for Library {
    type Target = Vec<FuzzyDir>;

    fn deref(&self) -> &Self::Target {
        &self.fdirs
    }
}

impl DerefMut for Library {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.fdirs
    }
}

impl core::fmt::Debug for Library {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let _ = writeln!(f, "Library:");
        for dir in &self.fdirs {
            writeln!(f, "    {},", dir.name)?;
        }
        writeln!(f, "")
    }
}

pub fn cache_path() -> Result<PathBuf, TapError> {
    let home_dir = PathBuf::from(env::var("HOME")?);
    let cache_dir = PathBuf::from(home_dir).join(".cache/tap");
    fs::create_dir_all(&cache_dir)?;
    let cache_path = PathBuf::from(cache_dir).join("data");

    Ok(cache_path)
}
