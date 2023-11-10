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

use crate::args;

lazy_static::lazy_static! {
    pub static ref COLOR_MAP: HashMap<String, Color> = default_palette();
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

pub fn fg() -> ColorStyle {
    ColorStyle::front(PALETTE["fg"])
}

pub fn hl() -> ColorStyle {
    ColorStyle::front(PALETTE["hl"])
}

pub fn prompt() -> ColorStyle {
    ColorStyle::front(PALETTE["prompt"])
}

pub fn header1() -> ColorStyle {
    ColorStyle::front(PALETTE["header"])
}

pub fn header2() -> ColorStyle {
    ColorStyle::front(PALETTE["header+"])
}

pub fn progress() -> ColorStyle {
    ColorStyle::front(PALETTE["progress"])
}

pub fn info() -> ColorStyle {
    ColorStyle::front(PALETTE["info"])
}

pub fn err() -> ColorStyle {
    ColorStyle::front(PALETTE["err"])
}

pub fn button() -> ColorStyle {
    ColorStyle::new(PALETTE["bg"], PALETTE["fg"])
}

fn create_palette() -> HashMap<String, Color> {
    // Get the default colors.
    let mut m = COLOR_MAP.to_owned();

    if args::term_color() {
        // Use terminal colors for foreground and background.
        for (_, value) in m.iter_mut() {
            *value = Color::TerminalDefault;
        }
    } else {
        // Update any user-defined colors.
        let (user_colors, term_bg) = args::user_colors();
        m.extend(user_colors);

        // Update background color with terminal color, if using.
        if term_bg {
            m.insert("bg".to_string(), Color::TerminalDefault);
        }
    }
    m
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
