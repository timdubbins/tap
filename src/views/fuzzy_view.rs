use cursive::event::{Event, EventResult, Key, MouseButton, MouseEvent};
use cursive::theme::Effect;
use cursive::view::{Resizable, View};
use cursive::views::LayerPosition;
use cursive::{Cursive, Printer, XY};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::app::curr_path;
use crate::fuzzy::*;
use crate::player::Player;
use crate::theme;
use crate::utils::*;
use crate::views::{utils::pop_layers_to_bottom, ErrorView, PlayerView};

#[derive(Clone)]
pub struct FuzzyView {
    // The text input to fuzzy match with.
    query: String,
    // The column of the text input cursor.
    cursor: usize,
    // The index of the selected item.
    selected: usize,
    // The vertical offset required to show `selected`.
    offset: usize,
    // The number of fuzzy matches.
    matches: usize,
    // The items to fuzzy search on.
    items: Vec<FuzzyItem>,
    // The size of the view.
    size: XY<usize>,
}

impl FuzzyView {
    pub fn new(items: Vec<FuzzyItem>) -> Self {
        FuzzyView {
            query: String::new(),
            cursor: 0,
            selected: 0,
            offset: 0,
            matches: items.len(),
            items,
            size: XY { x: 0, y: 0 },
        }
    }

    pub fn load(items: Vec<FuzzyItem>, siv: &mut Cursive) {
        // let size = siv.screen_size();
        siv.add_layer(FuzzyView::new(items).full_screen())
    }

    // Moves the selection down one row.
    fn move_down(&mut self) {
        if self.selected == 0 {
            return;
        }
        if self.selected == self.offset {
            self.offset -= 1;
        }
        self.selected -= 1;
    }

    // Moves the selection up one row.
    fn move_up(&mut self) {
        if self.selected == self.matches - 1 {
            return;
        }
        if self.selected - self.offset >= self.size.y - 3 {
            self.offset += 1;
        }
        self.selected += 1;
    }

    // Moves the selection up one page.
    fn page_up(&mut self) {
        if self.matches == 0 {
            return;
        }
        if self.selected + self.size.y - 3 <= self.matches - 1 {
            self.offset += self.size.y - 3;
            self.selected += self.size.y - 3;
        } else {
            self.selected = self.matches - 1;
        }
    }

    // Moves the selection down one page.
    fn page_down(&mut self) {
        if self.matches == 0 {
            return;
        }
        if self.selected + self.offset > self.size.y - 3 {
            self.offset -= self.size.y - 3;
            self.selected -= self.size.y - 3;
        } else {
            self.selected = 0;
        }
    }

    // Moves the selection to a random page.
    fn random_page(&mut self) {
        if self.items.len() <= self.size.y - 3 {
            return;
        }

        let pages = self.items.len() / (self.size.y - 3) + 1;
        let page = random(0..pages);
        let y = page * (self.size.y - 3);

        if y == self.offset {
            self.random_page();
        } else {
            self.clear();
            self.offset = y;
            self.selected = y;
        }
    }

    // Moves the cursor left one column.
    fn move_left(&mut self) {
        if self.cursor > 0 {
            let len = {
                let text = &self.query[0..self.cursor];
                text.graphemes(true).last().unwrap().len()
            };
            self.cursor -= len;
        }
    }

    // Moves the cursor right one column.
    fn move_right(&mut self) {
        if self.cursor < self.query.len() {
            let len = self.query[self.cursor..]
                .graphemes(true)
                .next()
                .unwrap()
                .len();
            self.cursor += len;
        }
    }

    // Deletes the character to the left of the cursor.
    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.move_left();
            self.delete()
        }
    }

    // Deletes the character to the right of the cursor.
    fn delete(&mut self) {
        if self.cursor == self.query.len() {
            self.update_list("");
        } else if self.cursor < self.query.len() {
            let len = self.query[self.cursor..]
                .graphemes(true)
                .next()
                .unwrap()
                .len();
            for _ in self.query.drain(self.cursor..self.cursor + len) {}
            self.update_list(&self.query.clone());
        }
    }

    // Inserts a character from user input to the right of the cursor.
    fn insert(&mut self, ch: char) {
        self.query.insert(self.cursor, ch);
        let shift = ch.len_utf8();
        self.cursor += shift;
        self.update_list(&self.query.to_owned());
    }

    // Removes the current fuzzy query.
    fn clear(&mut self) {
        self.query.clear();
        self.cursor = 0;
        self.update_list("");
    }

    // Runs the fuzzy matcher on the query.
    fn update_list(&mut self, pattern: &str) {
        if self.query.is_empty() {
            for (i, _) in self.items.clone().into_iter().enumerate() {
                self.items[i].weight = 1;
                self.items[i].indices.clear();
            }
            self.matches = self.items.len();
            self.selected = 0;
            self.offset = 0;
            return;
        }

        self.matches = self.fuzzy_match(pattern);
        self.sort();
        self.selected = 0;
        self.offset = 0;
    }

    // Sort the items by `weight` in descending order.
    fn sort(&mut self) {
        self.items.sort_by(|a, b| b.weight.cmp(&a.weight))
    }

    fn fuzzy_match(&mut self, pattern: &str) -> usize {
        let mut count = 0;
        let matcher = Box::new(SkimMatcherV2::default());
        for (i, item) in self.items.clone().into_iter().enumerate() {
            if let Some((weight, indices)) = matcher.fuzzy_indices(&item.display, pattern) {
                self.items[i].weight = weight;
                self.items[i].indices = indices;
                count += 1;
            } else {
                self.items[i].weight = 0;
                self.items[i].indices.clear();
            }
        }
        count
    }

    // The number of matched items over total items.
    fn count(&self) -> String {
        format!("{}/{} ", self.matches, self.items.len())
    }

    // Handle a fuzzy match being selected.
    fn on_select(&mut self) -> EventResult {
        // The fuzzy selected path.
        let selected = self.items[self.selected].path.to_owned();

        if has_child_dirs(&selected) {
            // Requires fuzzy matching on `selected`.
            self.clear();
            return EventResult::with_cb(move |siv| {
                FuzzyView::load(get_items(&selected), siv);
                siv.screen_mut().remove_layer(LayerPosition::FromFront(1));
            });
        }

        EventResult::with_cb(move |siv| {
            // The path of the current player.
            let current = curr_path(siv);

            if Some(selected.to_owned()).eq(&current) {
                // Don't reload the player if the selection hasn't changed.
                pop_layers_to_bottom(siv);
            } else {
                match Player::new(selected.to_owned()) {
                    Ok(player) => PlayerView::load(player, siv),
                    // Err(e) => self.error = Some(e.to_string()),
                    Err(e) => ErrorView::load(siv, e),
                }
            }
        })
    }

    fn mouse_select(&mut self, event: Event) -> EventResult {
        let mouse_y = event.mouse_position().unwrap_or_default().y;
        let available_y = self.size.y - 2;

        if mouse_y < 1 || mouse_y > available_y {
            return EventResult::Consumed(None);
        }

        if available_y + self.offset - mouse_y == self.selected {
            return self.on_select();
        } else {
            self.selected = available_y + self.offset - mouse_y;
            EventResult::Consumed(None)
        }
    }
}

impl View for FuzzyView {
    fn layout(&mut self, size: cursive::Vec2) {
        self.size = size;
    }

    fn draw(&self, p: &Printer) {
        // The size of the screen we can draw on.
        let (w, h) = (p.size.x, p.size.y);

        if h > 3 {
            // The first row of the list.
            let start_row = h - 3;
            // The number of visible rows.
            let visible = std::cmp::min(self.matches - self.offset, h - 2);

            for y in 0..visible {
                let index = y + self.offset;
                // The items are drawn in ascending order, starting on third row from bottom.
                let row = start_row - y;
                // Only draw items that have matches.
                if self.items[index].weight != 0 {
                    // Set the color depending on whether row is currently selected or not.
                    let (primary, highlight) = if row == start_row + self.offset - self.selected {
                        // Draw the symbol to show the currently selected item.
                        p.with_color(theme::yellow(), |p| p.print((0, row), ">"));
                        // The colors for the currently selected row.
                        (theme::white(), theme::green())
                    } else {
                        // The colors for the not selected row.
                        (theme::blue(), theme::white())
                    };
                    // Draw the item's display name.
                    p.with_color(primary, |p| {
                        p.print((2, row), self.items[index].display.as_str())
                    });
                    // Draw the fuzzy matched indices in a highlighting color.
                    for x in &self.items[index].indices {
                        let mut chars = self.items[index].display.chars();
                        p.with_effect(Effect::Bold, |p| {
                            p.with_color(highlight, |p| {
                                p.print(
                                    (x + 2, row),
                                    chars.nth(*x).unwrap_or_default().to_string().as_str(),
                                )
                            });
                        });
                    }
                }
            }

            // Draw the page count.
            p.with_color(theme::grey(), |p| {
                let page = self.selected / start_row;
                let pages = self.matches / start_row;
                let digits = page.checked_ilog10().unwrap_or(0) as usize
                    + pages.checked_ilog10().unwrap_or(0) as usize
                    + 2;
                let column = self.size.x - digits - 2;
                p.print((column, 0), format!(" {}/{}", page, pages).as_str());
            });
        }

        if h > 1 {
            // The last row we can draw on.
            let query_row = h - 1;

            // Draw the match count and some borders.
            p.with_color(theme::magenta(), |p| {
                let lines = std::cmp::min(self.matches / 4, h / 4);
                p.print_vline((w - 1, query_row - 1 - lines), lines, "│");
                p.print_hline((2, query_row - 1), w - 3, "─");
                p.print((2, query_row - 1), &self.count());
            });

            // Draw the text input area that shows the query.
            p.with_color(theme::inverted(), |p| {
                p.with_effect(Effect::Reverse, |p| {
                    p.print_hline((0, query_row), w, " ");
                    p.print((2, query_row), &self.query);
                });

                let c = if self.cursor == self.query.len() {
                    "_"
                } else {
                    &self.query[self.cursor..]
                        .graphemes(true)
                        .next()
                        .expect("should find a char")
                };
                let offset = self.query[..self.cursor].width();
                p.print((offset + 2, query_row), c);
            });
            // Draw the symbol to show the start of the text input area.
            p.with_color(theme::grey(), |p| p.print((0, query_row), ">"));
        }
    }

    // Keybindings for the fuzzy view.
    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char(ch) => self.insert(ch),
            Event::Key(Key::Enter) => return self.on_select(),

            Event::Key(Key::Esc)
            | Event::Mouse {
                event: MouseEvent::Press(MouseButton::Right),
                ..
            } => return on_cancel(),

            Event::Mouse {
                event: MouseEvent::Press(MouseButton::Left),
                ..
            } => return self.mouse_select(event),

            Event::Key(Key::Down)
            | Event::Mouse {
                event: MouseEvent::WheelDown,
                ..
            } => self.move_down(),

            Event::Key(Key::Up)
            | Event::Mouse {
                event: MouseEvent::WheelUp,
                ..
            } => self.move_up(),

            Event::Key(Key::PageUp) | Event::CtrlChar('h') => self.page_up(),
            Event::Key(Key::PageDown) | Event::CtrlChar('l') => self.page_down(),
            Event::CtrlChar('z') => self.random_page(),
            Event::Key(Key::Backspace) => self.backspace(),
            Event::Key(Key::Del) => self.delete(),
            Event::Key(Key::Left) => self.move_left(),
            Event::Key(Key::Right) => self.move_right(),
            Event::Key(Key::Home) => self.cursor = 0,
            Event::Key(Key::End) => self.cursor = self.query.len(),
            Event::CtrlChar('u') => self.clear(),
            _ => return EventResult::Ignored,
        }

        EventResult::Consumed(None)
    }
}

// Handle a fuzzy match being escaped.
fn on_cancel() -> EventResult {
    EventResult::with_cb(|siv| {
        if curr_path(siv).is_none() {
            siv.quit()
        } else {
            pop_layers_to_bottom(siv);
        }
    })
}
