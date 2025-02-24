use std::{
    collections::{hash_map::IntoIter, HashMap},
    iter::IntoIterator,
    ops::{Deref, DerefMut},
};

use {
    anyhow::{anyhow, bail},
    cursive::{
        theme::{
            BorderStyle,
            Color::{self, Rgb},
            ColorStyle, Palette,
            PaletteColor::{self, *},
            Theme as CursiveTheme,
        },
        With,
    },
};

use crate::TapError;

// A struct representing a theme, which maps UI elements to specific colors.
#[derive(Debug)]
pub struct Theme {
    // A mapping between element names and their corresponding colors.
    pub color_map: HashMap<String, Color>,
}

impl Theme {
    const COLOR_NAMES: [&'static str; 9] = [
        "fg", "bg", "hl", "prompt", "header_1", "header_2", "progress", "info", "err",
    ];

    pub fn validate_color(name: &str) -> bool {
        Self::COLOR_NAMES.contains(&name)
    }

    pub fn set_term_color(&mut self) {
        self.iter_mut()
            .for_each(|(_, value)| *value = Color::TerminalDefault);
    }

    pub fn set_term_bg(&mut self) {
        self.insert("bg".to_string(), Color::TerminalDefault);
    }
}

impl Deref for Theme {
    type Target = HashMap<String, Color>;

    fn deref(&self) -> &Self::Target {
        &self.color_map
    }
}

impl DerefMut for Theme {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.color_map
    }
}

impl TryFrom<String> for Theme {
    type Error = TapError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut color_map = HashMap::new();

        for pair in value.split(',').map(str::trim) {
            if let Some((key, value)) = pair.split_once('=') {
                let name = key.trim();
                let value = value.trim();

                if !Self::validate_color(name) {
                    bail!(
                        "Invalid color name '{}' for '--color <COLOR>'.\nAvailable names: 'fg', 'bg', 'hl', 'prompt', 'header_1', 'header_2', 'progress', 'info', 'err'",
                        name
                    );
                }

                let color = Color::parse(value).ok_or_else(|| {
                    anyhow!(
                        "Invalid color value '{}' for '--color <COLOR>'.\nExample values: 'red', 'light green', '#123456'",
                        value
                    )
                })?;

                color_map.insert(name.to_string(), color);
            }
        }

        let theme = Theme { color_map };

        Ok(theme)
    }
}

impl From<HashMap<String, String>> for Theme {
    fn from(value: HashMap<String, String>) -> Self {
        let color_map = value
            .into_iter()
            .filter_map(|(name, value)| {
                Theme::validate_color(&name)
                    .then(|| Color::parse(&value).map(|color| (name, color)))
                    .flatten()
            })
            .collect();

        Theme { color_map }
    }
}

impl IntoIterator for Theme {
    type Item = (String, Color);
    type IntoIter = IntoIter<String, Color>;

    fn into_iter(self) -> Self::IntoIter {
        self.color_map.into_iter()
    }
}

impl Default for Theme {
    fn default() -> Self {
        let mut m = HashMap::new();
        m.insert("fg".into(), Rgb(129, 161, 190)); // blue #81a1be
        m.insert("bg".into(), Rgb(31, 33, 29)); // black #1f211d
        m.insert("hl".into(), Rgb(197, 200, 198)); // white #c5c8c6
        m.insert("prompt".into(), Rgb(57, 54, 62)); // grey #39363e
        m.insert("header_1".into(), Rgb(181, 189, 104)); // green #b5bd68
        m.insert("header_2".into(), Rgb(240, 198, 116)); // yellow #f0c674
        m.insert("progress".into(), Rgb(178, 148, 187)); // magenta #b294bb
        m.insert("info".into(), Rgb(138, 190, 183)); // cyan #8abeb7
        m.insert("err".into(), Rgb(204, 102, 102)); // red #cc6666

        Self { color_map: m }
    }
}

impl From<&Theme> for CursiveTheme {
    fn from(theme: &Theme) -> Self {
        CursiveTheme {
            shadow: false,
            borders: BorderStyle::Simple,
            palette: Palette::default().with(|palette| {
                palette[Shadow] = theme.color_map["progress"];
                palette[Primary] = theme.color_map["hl"];
                palette[Secondary] = theme.color_map["fg"];
                palette[Tertiary] = theme.color_map["prompt"];
                palette[Background] = theme.color_map["bg"];
                palette[View] = theme.color_map["bg"];
                palette[TitlePrimary] = theme.color_map["header_1"];
                palette[TitleSecondary] = theme.color_map["header_2"];
                palette[Highlight] = theme.color_map["info"];
                palette[HighlightInactive] = theme.color_map["err"];
            }),
        }
    }
}

// A marker struct that provides predefined `ColorStyle` instances
// for various UI elements.
#[derive(Debug)]
pub struct ColorStyles;

impl ColorStyles {
    #[inline]
    pub fn fg() -> ColorStyle {
        ColorStyle::front(PaletteColor::Secondary)
    }

    #[inline]
    pub fn hl() -> ColorStyle {
        ColorStyle::front(PaletteColor::Primary)
    }

    #[inline]
    pub fn prompt() -> ColorStyle {
        ColorStyle::front(PaletteColor::Tertiary)
    }

    #[inline]
    pub fn header_1() -> ColorStyle {
        ColorStyle::front(PaletteColor::TitlePrimary)
    }

    #[inline]
    pub fn header_2() -> ColorStyle {
        ColorStyle::front(PaletteColor::TitleSecondary)
    }

    #[inline]
    pub fn progress() -> ColorStyle {
        ColorStyle::front(PaletteColor::Shadow)
    }

    #[inline]
    pub fn info() -> ColorStyle {
        ColorStyle::front(PaletteColor::Highlight)
    }

    #[inline]
    pub fn err() -> ColorStyle {
        ColorStyle::front(PaletteColor::HighlightInactive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_palette_uses_only_defined_names() {
        let palette = Theme::default().color_map;

        for name in palette.keys() {
            assert!(
                Theme::validate_color(name),
                "Palette contains an undefined color name: {}",
                name
            );
        }
    }
}
