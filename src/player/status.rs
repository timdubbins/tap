#[derive(Clone, PartialEq)]
pub enum PlayerStatus {
    Paused,
    Playing,
    Stopped,
}

pub trait StatusConversion {
    fn to_status(&self) -> PlayerStatus;
    fn to_u8(&self) -> u8;
}

impl StatusConversion for u8 {
    fn to_status(&self) -> PlayerStatus {
        match self {
            0 => PlayerStatus::Playing,
            1 => PlayerStatus::Paused,
            _ => PlayerStatus::Stopped,
        }
    }

    fn to_u8(&self) -> u8 {
        *self
    }
}

impl StatusConversion for PlayerStatus {
    fn to_u8(&self) -> u8 {
        match self {
            PlayerStatus::Playing => 0,
            PlayerStatus::Paused => 1,
            PlayerStatus::Stopped => 2,
        }
    }

    fn to_status(&self) -> PlayerStatus {
        self.clone()
    }
}
