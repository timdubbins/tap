use crate::utils::IntoInner;

use super::{BytesToStatus, PlayerStatus, StatusToBytes};

pub struct PlayerOpts {
    pub status: PlayerStatus,
    pub volume: u8,
    pub is_muted: bool,
    pub showing_volume: bool,
}

impl Default for PlayerOpts {
    fn default() -> Self {
        Self {
            status: PlayerStatus::Playing,
            volume: 100,
            is_muted: false,
            showing_volume: false,
        }
    }
}

impl Into<PlayerOpts> for (u8, u8, bool, bool) {
    fn into(self) -> PlayerOpts {
        PlayerOpts {
            status: self.0.from_u8(),
            volume: self.1,
            is_muted: self.2,
            showing_volume: self.3,
        }
    }
}

impl IntoInner for PlayerOpts {
    type T = (u8, u8, bool, bool);

    fn into_inner(self) -> Self::T {
        (
            self.status.to_u8(),
            self.volume,
            self.is_muted,
            self.showing_volume,
        )
    }
}
