#[derive(Clone, Debug, PartialEq)]
pub enum PlayerStatus {
    Paused,
    Playing,
    Stopped,
}

pub trait BytesToStatus {
    fn from_u8(&self) -> PlayerStatus;
}

pub trait StatusToBytes {
    fn to_u8(&self) -> u8;
}

impl BytesToStatus for u8 {
    fn from_u8(&self) -> PlayerStatus {
        match self {
            0 => PlayerStatus::Playing,
            1 => PlayerStatus::Paused,
            _ => PlayerStatus::Stopped,
        }
    }
}

impl StatusToBytes for PlayerStatus {
    fn to_u8(&self) -> u8 {
        match self {
            PlayerStatus::Playing => 0,
            PlayerStatus::Paused => 1,
            PlayerStatus::Stopped => 2,
        }
    }
}
