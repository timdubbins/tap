pub mod audio_file;
pub mod opts;
pub mod player;
pub mod status;

pub use self::{
    audio_file::{is_valid, AudioFile},
    opts::PlayerOpts,
    player::Player,
    status::{BytesToStatus, PlayerStatus, StatusToBytes},
};
