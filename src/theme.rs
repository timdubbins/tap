use cursive::theme::{Color, ColorStyle};

const FZF_THEME: &'static str = "--color=bg+:#3F3F3F,bg:#4B4B4B,border:#6B6B6B,spinner:#98BC99,hl:#719872,fg:#D9D9D9,header:#719872,info:#BDBB72,pointer:#E12672,marker:#E17899,fg+:#D9D9D9,preview-bg:#3F3F3F,prompt:#98BEDE,hl+:#98BC99";
const BLACK: Color = Color::Rgb(29, 31, 33); // #1d1f21
const WHITE: Color = Color::Rgb(197, 200, 198); // #c5c8c6
const RED: Color = Color::Rgb(204, 102, 102); // #cc6666
const GREEN: Color = Color::Rgb(181, 189, 104); // #b5bd68
const YELLOW: Color = Color::Rgb(240, 198, 116); // #f0c674
const BLUE: Color = Color::Rgb(129, 162, 190); // #81a2be
const MAGENTA: Color = Color::Rgb(178, 148, 187); // #b294bb

// const CYAN: Color = Color::Rgb(138, 190, 183); #8abeb7

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

// pub fn cyan() -> ColorStyle {
//     ColorStyle::new(CYAN, BLACK)
// }
