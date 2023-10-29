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
    "fg", "bg", "hl", "prompt", "header", "header+", "progress", "info", "err",
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

fn default_palette() -> HashMap<String, Color> {
    let mut m = HashMap::new();
    m.insert("fg".into(), Rgb(129, 162, 190)); // blue #81a2be
    m.insert("bg".into(), Rgb(31, 33, 29)); // black #1f211d
    m.insert("hl".into(), Rgb(197, 200, 198)); // white #c5c8c6
    m.insert("prompt".into(), Rgb(57, 54, 62)); // grey #39363e
    m.insert("header".into(), Rgb(181, 189, 104)); // green #b5bd68
    m.insert("header+".into(), Rgb(240, 198, 116)); // yellow #f0c674
    m.insert("progress".into(), Rgb(178, 148, 187)); // magenta #b294bb
    m.insert("info".into(), Rgb(138, 190, 183)); // cyan #8abeb7
    m.insert("err".into(), Rgb(204, 102, 102)); // red #cc6666
    m
}

fn create_palette() -> HashMap<String, Color> {
    // Define the default colors.
    let mut m = default_palette();

    // Override the default colors with any user-defined colors.
    let user_colors = crate::args::parse_user_colors();
    for (key, val) in user_colors.colors {
        m.insert(key, val);
    }

    // Terminal background overrides user-defined background.
    if user_colors.term_bg {
        m.insert("bg".into(), Color::TerminalDefault);
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
