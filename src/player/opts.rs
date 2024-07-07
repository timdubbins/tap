use crate::utils::IntoInner;

use super::PlayerStatus;

// Options for the player constructor.
#[derive(Debug)]
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
            status: PlayerStatus::from_u8(self.0),
            volume: self.1,
            is_muted: self.2,
            showing_volume: self.3,
        }
    }
}

//FIXME - choose Into or From impl
// impl From<(u8, u8, bool, bool)> for PlayerOpts {
//     fn from(val: (u8, u8, bool, bool)) -> Self {
//         PlayerOpts {
//             status: val.0.from_u8(),
//             volume: val.1,
//             is_muted: val.2,
//             showing_volume: val.3,
//         }
//     }
// }

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
