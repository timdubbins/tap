use std::collections::HashMap;

use cursive::{
    theme::{
        BorderStyle,
        Color::{self, Rgb},
        ColorStyle, Palette,
        PaletteColor::*,
        Theme,
    },
    With,
};

pub const COLOR_MAP: &'static [(&'static str, Color)] = &[
    ("fg", Rgb(129, 162, 190)),       // blue #81a2be
    ("bg", Rgb(31, 33, 29)),          // black #1f211d
    ("hl", Rgb(197, 200, 198)),       // white #c5c8c6
    ("prompt", Rgb(57, 54, 62)),      // grey #39363e
    ("header", Rgb(181, 189, 104)),   // green #b5bd68
    ("header+", Rgb(240, 198, 116)),  // yellow #f0c674
    ("progress", Rgb(178, 148, 187)), // magenta #b294bb
    ("info", Rgb(138, 190, 183)),     // cyan #8abeb7
    ("err", Rgb(204, 102, 102)),      // red #cc6666
];

lazy_static::lazy_static! {
    pub static ref PALETTE: HashMap<String, Color> = create_palette();
}

pub fn custom() -> Theme {
    Theme {
        shadow: false,
        borders: BorderStyle::Simple,
        palette: Palette::default().with(|palette| {
            palette[Background] = PALETTE["bg"];
            palette[View] = PALETTE["bg"];
            palette[Primary] = PALETTE["hl"];
            palette[TitlePrimary] = PALETTE["header"];
        }),
    }
}

pub fn color_style(key: &'static str) -> ColorStyle {
    ColorStyle::front(PALETTE[key])
}

pub fn button() -> ColorStyle {
    ColorStyle::new(PALETTE["bg"], PALETTE["fg"])
}

fn create_palette() -> HashMap<String, Color> {
    // Create the default colors.
    let mut m = HashMap::new();
    for (k, v) in COLOR_MAP {
        m.insert(k.to_string(), *v);
    }

    // Update any user-defined colors.
    let user_colors = crate::args::parse_user_colors();
    for (k, v) in user_colors.colors {
        m.insert(k, v);
    }

    // Update background color with terminal color, if using.
    if user_colors.term_bg {
        m.insert("bg".to_string(), Color::TerminalDefault);
    }

    m
}

pub struct UserColors {
    colors: Vec<(String, Color)>,
    term_bg: bool,
}

impl UserColors {
    pub fn new(colors: Vec<(String, Color)>, term_bg: bool) -> Self {
        UserColors { colors, term_bg }
    }
}
