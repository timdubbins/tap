use std::{
    cmp::{min, Ordering},
    path::PathBuf,
};

use cursive::{
    event::{Event, EventResult, EventTrigger, Key, MouseButton, MouseEvent},
    theme::Effect,
    view::Resizable,
    views::LayerPosition,
    Cursive, Printer, View, XY,
};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::config::{args, theme};
use crate::data::session_data::SessionData;
use crate::player::{PlayerBuilder, PlayerView};
use crate::utils::{self, InnerType};

use super::{create_items, load_error_view, FuzzyItem};

#[derive(Clone)]
pub struct FuzzyView {
    // The text input to fuzzy match with.
    query: String,
    // The column of the text input cursor.
    cursor: usize,
    // The index of the selected item.
    selected: usize,
    // The vertical offset required to show `selected`.
    offset_y: usize,
    // The number of fuzzy matches.
    matches: usize,
    // The items to fuzzy search on.
    items: Vec<FuzzyItem>,
    // The size of the view.
    size: XY<usize>,
}

impl FuzzyView {
    fn new(items: Vec<FuzzyItem>) -> Self {
        FuzzyView {
            query: String::new(),
            cursor: 0,
            selected: 0,
            offset_y: 0,
            matches: items.len(),
            items,
            // available_y: 0,
            size: XY { x: 0, y: 0 },
        }
    }

    // Loads a new FuzzyView from the provided items. Providing a `key` will
    // pre-match the results using the char.
    pub fn load(items: Vec<FuzzyItem>, key: Option<char>, siv: &mut Cursive) {
        let mut fuzzy = FuzzyView::new(items);

        if let Some(key) = key {
            fuzzy.insert(key.to_ascii_lowercase());
        }

        siv.add_layer(fuzzy.full_screen());
        remove_layer(siv);
    }

    // Moves the selection down one row.
    fn move_down(&mut self) {
        if self.selected == 0 {
            return;
        }
        if self.selected == self.offset_y {
            self.offset_y -= 1;
        }
        self.selected -= 1;
    }

    // Moves the selection up one row.
    fn move_up(&mut self) {
        if self.selected == self.matches - 1 {
            return;
        }

        let last_visible_row = self.visible_rows().1;

        if self.selected - self.offset_y > last_visible_row - 2 {
            self.offset_y += 1;
        }
        self.selected += 1;
    }

    // Moves the selection up one page.
    fn page_up(&mut self) {
        if self.matches == 0 {
            return;
        }

        let last_visible_row = self.visible_rows().1;

        if self.selected + last_visible_row < self.matches - 1 {
            self.offset_y += last_visible_row;
            self.selected += last_visible_row;
        } else {
            self.selected = self.matches - 1;
            if self.offset_y + last_visible_row <= self.selected {
                self.offset_y += last_visible_row;
            }
        }
    }

    // Moves the selection down one page.
    fn page_down(&mut self) {
        if self.matches == 0 {
            return;
        }

        // let available_y = self.available_y();
        let last_visible_row = self.visible_rows().1;

        self.selected = if self.selected >= last_visible_row {
            self.selected - last_visible_row
        } else {
            0
        };

        self.offset_y = if self.offset_y >= last_visible_row {
            self.offset_y - last_visible_row
        } else {
            0
        };
    }

    // Moves the selection to a random page.
    fn random_page(&mut self) {
        if let Some(start_row) = self.size.y.checked_sub(3) {
            if start_row < 1 {
                return;
            }

            if self.matches <= start_row {
                return;
            }

            let pages = self.matches / start_row;
            let page = utils::random(0..pages);
            let y = page * (start_row);

            self.clear();
            self.offset_y = y;
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
        match self.cursor.cmp(&self.query.len()) {
            Ordering::Equal => {
                self.update_list("");
            }
            Ordering::Less => {
                let len = self.query[self.cursor..]
                    .graphemes(true)
                    .next()
                    .unwrap()
                    .len();
                for _ in self.query.drain(self.cursor..self.cursor + len) {}
                self.update_list(&self.query.clone());
            }
            _ => (),
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
            self.offset_y = 0;
            return;
        }

        self.matches = self.fuzzy_match(pattern);
        self.sort();
        self.selected = 0;
        self.offset_y = 0;
    }

    // Sort the items by `weight` in descending order.
    fn sort(&mut self) {
        self.items.sort_by(|a, b| b.weight.cmp(&a.weight))
    }

    // Computes the weights for the items on fuzzy matching with the query.
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

    // Handles a fuzzy match being selected.
    fn on_select(&mut self) -> EventResult {
        if self.items.is_empty() {
            return EventResult::with_cb(|siv| {
                let err = anyhow::Error::msg("Nothing to select!");
                load_error_view(siv, err)
            });
        }

        let item = self.items[self.selected].to_owned();

        EventResult::with_cb(move |siv| {
            if item.child_count == 0 {
                select_player(item.to_owned(), siv);
            } else {
                let items = create_items(&item.path).expect("should always exist");

                if items.len() == 1 {
                    let item = items.first().unwrap();

                    if item.has_audio && item.child_count == 0 {
                        return select_player(item.to_owned(), siv);
                    }
                }

                FuzzyView::load(items, None, siv);
            }
        })
    }

    fn visible_rows(&self) -> (usize, usize) {
        let last_visible_row = if self.size.y > 2 { self.size.y - 2 } else { 0 };
        let first_visible_row = self.size.y - 1 - min(last_visible_row, self.matches);

        (first_visible_row, last_visible_row)
    }

    // Handles a selection from mouse input.
    fn mouse_select(&mut self, position: XY<usize>) -> EventResult {
        let (first_visible_row, last_visible_row) = self.visible_rows();

        if position.y < first_visible_row || position.y > last_visible_row {
            return EventResult::Consumed(None);
        }

        let next_selected = last_visible_row + self.offset_y - position.y;

        if next_selected == self.selected {
            self.on_select()
        } else {
            self.selected = next_selected;
            EventResult::Consumed(None)
        }
    }

    // Loads a fuzzy view for the parent of the current directory.
    fn parent(&self) -> EventResult {
        let mut parent = match self.items.first() {
            Some(parent) => parent.path.to_owned(),
            None => return EventResult::Ignored,
        };

        let root = args::search_root();
        if parent != root {
            parent.pop();
            if parent != root {
                parent.pop();
            }
        }

        EventResult::with_cb(move |siv| {
            if let Ok(items) = create_items(&parent) {
                FuzzyView::load(items, None, siv);
            }
        })
    }

    // Opens the current selected item in the preferred file manager.
    fn open_file_manager(&self) {
        if self.selected < self.items.len() {
            let path = self.items[self.selected].path.to_owned();
            _ = utils::open_file_manager(path);
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
            let visible_rows = min(self.matches - self.offset_y, h - 2);

            for y in 0..visible_rows {
                let index = y + self.offset_y;
                // The items are drawn in ascending order, starting on third row from bottom.
                let row = start_row - y;
                // Only draw items that have matches.
                if self.items[index].weight != 0 {
                    // Set the color depending on whether row is currently selected or not.
                    let (primary, highlight) = if row + self.selected == start_row + self.offset_y {
                        // Draw the symbol to show the currently selected item.
                        p.with_color(theme::header_2(), |p| p.print((0, row), ">"));
                        // The colors for the currently selected row.
                        (theme::hl(), theme::header_1())
                    } else {
                        // The colors for the not selected row.
                        (theme::fg(), theme::hl())
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
            if self.matches > 0 {
                p.with_color(theme::prompt(), |p| {
                    let rows_per_page = h - 2;
                    let current_page = self.selected / rows_per_page;
                    let total_pages = (self.matches + rows_per_page - 1) / rows_per_page - 1;
                    let formatted_page_count = format!(" {}/{}", current_page, total_pages);
                    let start_column = self.size.x - formatted_page_count.chars().count();
                    p.print((start_column, 0), formatted_page_count.as_str());
                });
            }
        }

        if h > 1 {
            // The last row we can draw on.
            let query_row = h - 1;

            // Draw the match count and some borders.
            p.with_color(theme::progress(), |p| {
                let lines = min(self.matches / 4, h / 4);
                p.print_vline((w - 1, query_row - 1 - lines), lines, "│");
                p.print_hline((2, query_row - 1), w - 3, "─");
                p.print((2, query_row - 1), &self.count());
            });

            // Draw the text input area that shows the query.
            p.with_color(theme::hl(), |p| {
                p.print_hline((0, query_row), w, " ");
                p.print((2, query_row), &self.query);
            });

            let c = if self.cursor == self.query.len() {
                "_"
            } else {
                self.query[self.cursor..]
                    .graphemes(true)
                    .next()
                    .expect("should find a char")
            };
            let offset = self.query[..self.cursor].width();
            p.with_effect(Effect::Reverse, |p| {
                p.print((offset + 2, query_row), c);
            });

            // Draw the symbol to show the start of the text input area.
            p.with_color(theme::prompt(), |p| p.print((0, query_row), ">"));
        }
    }

    // Keybindings for the fuzzy view.
    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char(ch) => self.insert(ch),
            Event::Key(Key::Enter) => return self.on_select(),
            Event::Key(Key::Esc) => return on_cancel(),
            Event::Key(Key::Down) => self.move_down(),
            Event::Key(Key::Up) => self.move_up(),
            Event::Key(Key::PageUp) | Event::CtrlChar('h') => self.page_up(),
            Event::Key(Key::PageDown) | Event::CtrlChar('l') => self.page_down(),
            Event::CtrlChar('r') => self.random_page(),
            Event::Key(Key::Backspace) => self.backspace(),
            Event::Key(Key::Del) => self.delete(),
            Event::Key(Key::Left) => self.move_left(),
            Event::Key(Key::Right) => self.move_right(),
            Event::Key(Key::Home) => self.cursor = 0,
            Event::Key(Key::End) => self.cursor = self.query.len(),
            Event::CtrlChar('u') => self.clear(),
            Event::CtrlChar('p') => return self.parent(),
            Event::CtrlChar('o') => self.open_file_manager(),

            Event::Mouse {
                event, position, ..
            } => match event {
                MouseEvent::Press(MouseButton::Right) => return on_cancel(),
                MouseEvent::Press(MouseButton::Left) => return self.mouse_select(position),
                MouseEvent::WheelDown => self.move_down(),
                MouseEvent::WheelUp => self.move_up(),
                _ => (),
            },
            _ => (),
        }
        EventResult::Consumed(None)
    }
}

pub fn fuzzy_finder(event: &Event, items: &Vec<FuzzyItem>) -> Option<EventResult> {
    let key = event.char();
    let (items, key) = match key {
        Some('A'..='Z') => (super::key_items(key, items), key),
        Some('a') => (super::non_leaf_items(items), None),
        Some('s') => (super::audio_items(items), None),
        _ => match event.f_num() {
            Some(depth) => (super::depth_items(depth, items), None),
            None => (items.to_owned(), None),
        },
    };
    Some(EventResult::with_cb(move |siv| {
        FuzzyView::load(items.to_owned(), key, siv)
    }))
}

// Trigger for the fuzzy-finder callbacks.
pub fn trigger() -> EventTrigger {
    EventTrigger::from_fn(|event| {
        matches!(
            event,
            Event::Key(Key::Tab)
                | Event::Char('A'..='Z')
                | Event::CtrlChar('a')
                | Event::CtrlChar('s')
                | Event::Key(Key::F1)
                | Event::Key(Key::F2)
                | Event::Key(Key::F3)
                | Event::Key(Key::F4)
                | Event::Mouse {
                    event: MouseEvent::Press(MouseButton::Middle),
                    ..
                }
        )
    })
}

fn select_player(item: FuzzyItem, siv: &mut Cursive) {
    let selected = Some(item.path);
    let current = current_path(siv);

    match PlayerBuilder::FuzzyFinder.from(selected.to_owned(), siv) {
        Ok(player) => {
            // Don't reload the player if the selection hasn't changed.
            if selected.eq(&current) {
                siv.pop_layer();
            } else {
                PlayerView::load(player, siv);
            }
        }
        Err(e) => load_error_view(siv, e),
    }
}

// Handle a fuzzy match being escaped.
fn on_cancel() -> EventResult {
    EventResult::with_cb(|siv| {
        if current_path(siv).is_none() {
            siv.quit()
        } else {
            siv.pop_layer();
        }
    })
}

// The path of the current player, if any.
pub fn current_path(siv: &mut Cursive) -> Option<PathBuf> {
    match siv.user_data::<InnerType<SessionData>>() {
        // match siv.user_data::<InnerType<UserData>>() {
        Some((_, _, queue)) => queue.get(1).map(|(p, _)| p.to_owned()),
        None => None,
    }
}

// Pops views from the view stack until there are only two remaining:
// the current FuzzyView and the underlying PlayerView.
fn remove_layer(siv: &mut Cursive) {
    while siv.screen().len() > 2 {
        siv.screen_mut().remove_layer(LayerPosition::FromFront(1));
    }
}
