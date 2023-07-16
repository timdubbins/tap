use cursive::theme::{Color, ColorStyle};

// The colors to use with fzf.
pub const FZF_THEME: &'static str = "--color=bg+:#131415,bg:#131415,\
border:#b294bb,spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,\
info:#b294bb,pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,preview-bg:#D9D9D9,\
prompt:#616161,hl+:#b9ca4a";

// The colors to use with sk.
pub const SK_THEME: &'static str = "--color=dark,border:#b294bb,\
spinner:#cc6666,hl:#c5c8c6,fg:#81a2be,header:#b5bd68,info:#b294bb,\
pointer:#f0c674,marker:#8abeb7,fg+:#c5c8c6,prompt:#616161,hl+:#b9ca4a";

const BLACK: Color = Color::Rgb(31, 33, 29); // #1f211d
const WHITE: Color = Color::Rgb(197, 200, 198); // #c5c8c6
const RED: Color = Color::Rgb(204, 102, 102); // #cc6666
const GREEN: Color = Color::Rgb(181, 189, 104); // #b5bd68
const YELLOW: Color = Color::Rgb(240, 198, 116); // #f0c674
const BLUE: Color = Color::Rgb(129, 162, 190); // #81a2be
const MAGENTA: Color = Color::Rgb(178, 148, 187); // #b294bb
const CYAN: Color = Color::Rgb(138, 190, 183); // #8abeb7

pub fn white() -> ColorStyle {
    ColorStyle::new(WHITE, BLACK)
}

pub fn red() -> ColorStyle {
    ColorStyle::new(RED, BLACK)
}
pub fn green() -> ColorStyle {
    ColorStyle::new(GREEN, BLACK)
}

pub fn yellow() -> ColorStyle {
    ColorStyle::new(YELLOW, BLACK)
}

pub fn blue() -> ColorStyle {
    ColorStyle::new(BLUE, BLACK)
}

pub fn magenta() -> ColorStyle {
    ColorStyle::new(MAGENTA, BLACK)
}

pub fn cyan() -> ColorStyle {
    ColorStyle::new(CYAN, BLACK)
}
