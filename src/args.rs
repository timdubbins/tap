use std::env::current_dir;
use std::io::Error;
use std::path::PathBuf;

use clap::Parser;

use crate::mode::Mode;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    path: Option<PathBuf>,

    #[clap(hide(true), default_value = None, long)]
    mode: Option<u8>,

    #[clap[hide(true), default_value = None, long]]
    initial_path: Option<String>,
}

impl Args {
    pub fn first_run() -> bool {
        match Args::parse().mode {
            Some(_) => false,
            None => true,
        }
    }

    pub fn parse_args() -> Result<(Mode, PathBuf), Error> {
        let path = match Args::parse().path {
            Some(p) => p,
            None => current_dir()?,
        };

        Ok((Args::parse_mode(&path), path))
    }

    pub fn get_path_arg() -> String {
        match Args::parse().initial_path {
            Some(p) => p,
            None => Args::parse()
                .path
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap(),
        }
    }

    fn parse_mode(path: &PathBuf) -> Mode {
        println!("parse mode path: {:?}", path);
        match Args::parse().mode {
            None => Mode::get_mode(path),
            Some(0) => Mode::NoFuzzyCurrentDir,
            Some(1) => Mode::NoFuzzyPathArg,
            Some(2) => Mode::FuzzyCurrentDir,
            Some(3) => Mode::FuzzyPathArg,
            Some(4_u8..=u8::MAX) => panic!("invalid argument"),
        }
    }
}
