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

pub const COLOR_KEYS: &'static [&'static str] = &[
    "bg", "hl", "track", "prompt", "artist", "album", "bar", "status", "stop",
];

lazy_static::lazy_static! {
    static ref PALETTE: HashMap<String, Color> = create_palette();
}

pub fn custom() -> Theme {
    Theme {
        shadow: false,
        borders: BorderStyle::Simple,
        palette: Palette::default().with(|palette| {
            palette[Background] = PALETTE["bg"];
            palette[View] = PALETTE["bg"];
            palette[Primary] = PALETTE["hl"];
            palette[TitlePrimary] = PALETTE["artist"];
        }),
    }
}

pub fn color_style(key: &'static str) -> ColorStyle {
    ColorStyle::front(PALETTE[key])
}

pub fn button() -> ColorStyle {
    ColorStyle::new(PALETTE["bg"], PALETTE["track"])
}

fn create_palette() -> HashMap<String, Color> {
    // Define the default colors.
    let mut m = HashMap::new();
    m.insert("bg".into(), Rgb(31, 33, 29)); // black #1f211d
    m.insert("hl".into(), Rgb(197, 200, 198)); // white #c5c8c6
    m.insert("track".into(), Rgb(129, 162, 190)); // blue #81a2be
    m.insert("prompt".into(), Rgb(57, 54, 62)); // grey #39363e
    m.insert("artist".into(), Rgb(181, 189, 104)); // green #b5bd68
    m.insert("album".into(), Rgb(240, 198, 116)); // yellow #f0c674
    m.insert("bar".into(), Rgb(178, 148, 187)); // magenta #b294bb
    m.insert("status".into(), Rgb(138, 190, 183)); // cyan #8abeb7
    m.insert("stop".into(), Rgb(204, 102, 102)); // red #cc6666

    // Override the default colors with any user-defined colors.
    let user_colors = crate::args::parse_user_colors();
    for (key, val) in user_colors.colors {
        m.insert(key, val);
    }

    // Terminal background overrides user-defined background.
    if user_colors.use_term_bg {
        m.insert("bg".into(), Color::TerminalDefault);
    }

    m
}

pub struct UserColors {
    colors: Vec<(String, Color)>,
    use_term_bg: bool,
}

impl UserColors {
    pub fn new(colors: Vec<(String, Color)>, use_term_bg: bool) -> Self {
        UserColors {
            colors,
            use_term_bg,
        }
    }
}
