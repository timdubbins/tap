use crate::utils::IntoInner;

use super::{BytesToStatus, PlayerStatus, StatusToBytes};

pub struct PlayerOpts {
    pub status: PlayerStatus,
    pub volume: u8,
    pub is_muted: bool,
}

impl Default for PlayerOpts {
    fn default() -> Self {
        Self {
            status: PlayerStatus::Playing,
            volume: 100,
            is_muted: false,
        }
    }
}

impl Into<PlayerOpts> for (u8, u8, bool) {
    fn into(self) -> PlayerOpts {
        PlayerOpts {
            status: self.0.from_u8(),
            volume: self.1,
            is_muted: self.2,
        }
    }
}

impl IntoInner for PlayerOpts {
    type T = (u8, u8, bool);

    fn into_inner(self) -> Self::T {
        (self.status.to_u8(), self.volume, self.is_muted)
    }
}
