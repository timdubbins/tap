#[derive(Clone, Debug, PartialEq)]
pub enum PlayerStatus {
    Paused,
    Playing,
    Stopped,
}

impl PlayerStatus {
    // Convert from a byte (u8) to PlayerStatus.
    pub fn from_u8(byte: u8) -> Self {
        match byte {
            0 => PlayerStatus::Playing,
            1 => PlayerStatus::Paused,
            // For any other value, default to Stopped
            _ => PlayerStatus::Stopped,
        }
    }

    // Convert from PlayerStatus to a byte (u8).
    pub fn to_u8(&self) -> u8 {
        match self {
            PlayerStatus::Playing => 0,
            PlayerStatus::Paused => 1,
            PlayerStatus::Stopped => 2,
        }
    }
}
