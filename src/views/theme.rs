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

lazy_static::lazy_static! {
    static ref BLACK: Color = find_bg();
}

const WHITE: Color = Rgb(197, 200, 198); // #c5c8c6
const RED: Color = Rgb(204, 102, 102); // #cc6666
const GREEN: Color = Rgb(181, 189, 104); // #b5bd68
const YELLOW: Color = Rgb(240, 198, 116); // #f0c674
const BLUE: Color = Rgb(129, 162, 190); // #81a2be
const MAGENTA: Color = Rgb(178, 148, 187); // #b294bb
const CYAN: Color = Rgb(138, 190, 183); // #8abeb7
const GREY: Color = Rgb(57, 54, 62); // #39363e

pub fn custom() -> Theme {
    Theme {
        shadow: false,
        borders: BorderStyle::Simple,
        palette: Palette::default().with(|palette| {
            palette[Background] = *BLACK;
            palette[View] = *BLACK;
            palette[Primary] = WHITE;
            palette[TitlePrimary] = GREEN;
        }),
    }
}

pub fn button() -> ColorStyle {
    ColorStyle::new(*BLACK, BLUE)
}

pub fn grey() -> ColorStyle {
    ColorStyle::front(GREY)
}

pub fn white() -> ColorStyle {
    ColorStyle::front(WHITE)
}

pub fn red() -> ColorStyle {
    ColorStyle::front(RED)
}

pub fn green() -> ColorStyle {
    ColorStyle::front(GREEN)
}

pub fn yellow() -> ColorStyle {
    ColorStyle::front(YELLOW)
}

pub fn blue() -> ColorStyle {
    ColorStyle::front(BLUE)
}

pub fn magenta() -> ColorStyle {
    ColorStyle::front(MAGENTA)
}

pub fn cyan() -> ColorStyle {
    ColorStyle::front(CYAN)
}

fn find_bg() -> Color {
    match crate::args::Args::use_default_bg() {
        true => Color::TerminalDefault,
        false => Rgb(31, 33, 29), // BLACK: #1f211d
    }
}
