use std::collections::HashMap;

use cursive::{
    theme::{
        BorderStyle,
        Color::{self, Rgb},
        ColorStyle, Palette,
        PaletteColor::{self, *},
        Theme,
    },
    With,
};

use super::config::Config;

const COLOR_NAMES: [&str; 9] = [
    "fg", "bg", "hl", "prompt", "header_1", "header_2", "progress", "info", "err",
];

pub fn validate_color(name: &str) -> bool {
    COLOR_NAMES.contains(&name)
}

pub fn custom(colors: &HashMap<String, Color>) -> Theme {
    Theme {
        shadow: false,
        borders: BorderStyle::Simple,
        palette: Palette::default().with(|palette| {
            palette[Shadow] = colors["progress"];
            palette[Primary] = colors["hl"];
            palette[Secondary] = colors["fg"];
            palette[Tertiary] = colors["prompt"];
            palette[Background] = colors["bg"];
            palette[View] = colors["bg"];
            palette[TitlePrimary] = colors["header_1"];
            palette[TitleSecondary] = colors["header_2"];
            palette[Highlight] = colors["info"];
            palette[HighlightInactive] = colors["err"];
        }),
    }
}

pub fn fg() -> ColorStyle {
    ColorStyle::front(PaletteColor::Secondary)
}

pub fn hl() -> ColorStyle {
    ColorStyle::front(PaletteColor::Primary)
}

pub fn prompt() -> ColorStyle {
    ColorStyle::front(PaletteColor::Tertiary)
}

pub fn header_1() -> ColorStyle {
    ColorStyle::front(PaletteColor::TitlePrimary)
}

pub fn header_2() -> ColorStyle {
    ColorStyle::front(PaletteColor::TitleSecondary)
}

pub fn progress() -> ColorStyle {
    ColorStyle::front(PaletteColor::Shadow)
}

pub fn info() -> ColorStyle {
    ColorStyle::front(PaletteColor::Highlight)
}

pub fn err() -> ColorStyle {
    ColorStyle::front(PaletteColor::HighlightInactive)
}

pub fn inverted() -> ColorStyle {
    ColorStyle::new(PaletteColor::Background, PaletteColor::Secondary)
}

pub fn parse_colors(args_colors: Vec<(String, Color)>, mut config: Config) -> Config {
    let mut palette = default_palette();

    if !config.use_default_palette {
        if config.use_term_default && args_colors.is_empty() {
            // Use terminal colors for foreground and background.
            for (_, value) in palette.iter_mut() {
                *value = Color::TerminalDefault;
            }
        } else {
            // Update any user-defined colors from config file.
            palette.extend(config.colors);

            // Update any user-defined colors from command args.
            palette.extend(args_colors);

            // Update background color with terminal color, if using.
            if config.use_term_bg {
                palette.insert("bg".to_string(), Color::TerminalDefault);
            }
        }
    }

    config.colors = palette;
    config
}

fn default_palette() -> HashMap<String, Color> {
    let mut m = HashMap::new();
    m.insert("fg".into(), Rgb(129, 162, 190)); // blue #81a2be
    m.insert("bg".into(), Rgb(31, 33, 29)); // black #1f211d
    m.insert("hl".into(), Rgb(197, 200, 198)); // white #c5c8c6
    m.insert("prompt".into(), Rgb(57, 54, 62)); // grey #39363e
    m.insert("header_1".into(), Rgb(181, 189, 104)); // green #b5bd68
    m.insert("header_2".into(), Rgb(240, 198, 116)); // yellow #f0c674
    m.insert("progress".into(), Rgb(178, 148, 187)); // magenta #b294bb
    m.insert("info".into(), Rgb(138, 190, 183)); // cyan #8abeb7
    m.insert("err".into(), Rgb(204, 102, 102)); // red #cc6666
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_palette_uses_only_defined_names() {
        let palette = default_palette();
        let defined_names = COLOR_NAMES.iter().collect::<std::collections::HashSet<_>>();

        for key in palette.keys() {
            assert!(
                defined_names.contains(&key.as_str()),
                "Palette contains an undefined color name: {}",
                key
            );
        }

        assert_eq!(
            palette.len(),
            COLOR_NAMES.len(),
            "Palette size does not match the number of defined color names"
        );
    }
}
