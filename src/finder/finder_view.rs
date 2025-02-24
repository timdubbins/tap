use std::{cmp::min, time::Instant};

use cursive::event::Key;
use once_cell::sync::Lazy;

use {
    anyhow::anyhow,
    cursive::{
        event::{Event, EventResult, MouseButton, MouseEvent},
        theme::Effect,
        view::{Nameable, Resizable},
        CbSink, Cursive, Printer, View, XY,
    },
    rand::{seq::SliceRandom, thread_rng},
    unicode_segmentation::UnicodeSegmentation,
    unicode_width::UnicodeWidthStr,
};

use crate::{
    config::ColorStyles,
    finder::{ErrorView, Finder, FuzzyDir, Library, LibraryFilter},
    player::{Player, PlayerView, Playlist},
};

static FRAMES: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃", "▂",
    ]
});

// A struct representing the view and state of the Finder.
pub struct FinderView {
    // The finder instance, which performs fuzzy searches and manages the results.
    finder: Finder,
    // The column of the text input cursor.
    cursor: usize,
    // The index of the selected directory in the search results.
    selected_index: usize,
    // Tracks the timestamp when initialization started.
    // Set to `Some(Instant::now())` when the struct is first initialized.
    // Set to `None` once initialization is complete and never used afterward.
    init_timestamp: Option<Instant>,
    // The vertical offset needed to ensure the selected directory is visible.
    offset_y: usize,
    // The dimensions of the view, in cells.
    size: XY<usize>,
    // A sender for scheduling callbacks to be executed by the Cursive root.
    cb_sink: CbSink,
}

impl FinderView {
    pub fn new(finder: Finder, cb_sink: CbSink) -> Self {
        FinderView {
            finder,
            cb_sink,
            init_timestamp: None,
            cursor: 0,
            selected_index: 0,
            offset_y: 0,
            size: XY { x: 0, y: 0 },
        }
    }

    pub fn load(filter: LibraryFilter) -> Option<EventResult> {
        Some(EventResult::with_cb(move |siv: &mut Cursive| {
            let library = {
                let base_library = siv
                    .user_data::<Library>()
                    .expect("Library should be set in user_data");

                let mut library = match &filter {
                    LibraryFilter::Parent(child) => base_library.parent_of(child.clone()),
                    _ => base_library.apply_filter(&filter),
                };

                library.shuffle(&mut thread_rng());
                library
            };

            let finder = Finder::new(library);
            let cb_sink = siv.cb_sink().clone();
            let mut finder_view = FinderView::new(finder, cb_sink);

            if let LibraryFilter::ByKey(key) = filter {
                finder_view.insert(key.to_ascii_lowercase(), false);
            }

            Self::remove(siv);
            siv.add_layer(finder_view.with_name(super::ID).full_screen());
        }))
    }

    // Update the internal library and recalc fuzzy matches.
    pub fn update_library(&mut self, batch: &mut Vec<FuzzyDir>) {
        self.finder.library.fdirs.append(batch);
        self.finder.update(self.finder.query.clone());
    }

    // Replace the internal library and recalc fuzzy matches.
    pub fn set_library(&mut self, library: Library) {
        self.finder.library = library;
        self.finder.update(self.finder.query.clone());
    }

    pub fn by_depth(event: &Event) -> Option<EventResult> {
        let depth = event.f_num().expect("event should be usize");
        Self::load(LibraryFilter::ByDepth(depth))
    }

    pub fn by_key(event: &Event) -> Option<EventResult> {
        let key = event.char().expect("event should be char");
        Self::load(LibraryFilter::ByKey(key))
    }

    pub fn all(_: &Event) -> Option<EventResult> {
        Self::load(LibraryFilter::Unfiltered)
    }

    pub fn by_artist(_: &Event) -> Option<EventResult> {
        Self::load(LibraryFilter::ByArtist)
    }

    pub fn by_album(_: &Event) -> Option<EventResult> {
        Self::load(LibraryFilter::ByAlbum)
    }

    pub fn parent(_: &Event) -> Option<EventResult> {
        Some(EventResult::with_cb(move |siv: &mut Cursive| {
            let child = siv
                .call_on_name(super::ID, |finder_view: &mut FinderView| {
                    finder_view.selected_dir()
                })
                .unwrap_or_else(|| {
                    siv.call_on_name(crate::player::ID, |player_view: &mut PlayerView| {
                        player_view.current_dir().clone()
                    })
                });

            if let Some(event) = Self::load(LibraryFilter::Parent(child)) {
                event.process(siv);
            }
        }))
    }

    fn next(&mut self) {
        if self.selected_index == 0 {
            return;
        }
        if self.selected_index == self.offset_y {
            self.offset_y -= 1;
        }
        self.selected_index -= 1;
    }

    pub fn previous(&mut self) {
        if self.selected_index == self.finder.matches - 1 {
            return;
        }
        let last_visible_row = self.visible_rows().1;

        if self.selected_index - self.offset_y > last_visible_row - 2 {
            self.offset_y += 1;
        }
        self.selected_index += 1;
    }

    fn page_up(&mut self) {
        if self.finder.matches == 0 {
            return;
        }

        let last_visible_row = self.visible_rows().1;

        if self.selected_index + last_visible_row < self.finder.matches - 1 {
            self.offset_y += last_visible_row;
            self.selected_index += last_visible_row;
        } else {
            self.selected_index = self.finder.matches - 1;
            if self.offset_y + last_visible_row <= self.selected_index {
                self.offset_y += last_visible_row;
            }
        }
    }

    fn page_down(&mut self) {
        if self.finder.matches == 0 {
            return;
        }
        let last_visible_row = self.visible_rows().1;

        self.selected_index = if self.selected_index >= last_visible_row {
            self.selected_index - last_visible_row
        } else {
            0
        };

        self.offset_y = if self.offset_y >= last_visible_row {
            self.offset_y - last_visible_row
        } else {
            0
        };
    }

    fn cursor_left(&mut self) {
        if self.cursor > 0 {
            let len = {
                let text = &self.finder.query[0..self.cursor];
                text.graphemes(true).last().unwrap().len()
            };
            self.cursor -= len;
        }
    }

    fn cursor_right(&mut self) {
        if self.cursor < self.finder.query.len() {
            let len = self.finder.query[self.cursor..]
                .graphemes(true)
                .next()
                .unwrap()
                .len();
            self.cursor += len;
        }
    }

    fn update_search_results(&mut self, query: String) {
        self.finder.update(query);
        self.selected_index = 0;
        self.offset_y = 0;
    }

    fn delete(&mut self) {
        if self.cursor < self.finder.query.len() {
            let mut query = self.finder.query.clone();
            let len = query[self.cursor..].graphemes(true).next().unwrap().len();
            query.drain(self.cursor..self.cursor + len);
            self.update_search_results(query);
        } else if self.finder.query.len() == 0 {
            self.clear();
        }
    }

    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor_left();
            self.delete()
        } else {
            _ = self.cb_sink.send(Box::new(|siv| {
                Self::load(LibraryFilter::Unfiltered).map(|e| e.process(siv));
            }))
        }
    }

    fn insert(&mut self, ch: char, apply_matching: bool) {
        let mut query = self.finder.query.clone();
        query.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();

        if apply_matching {
            self.update_search_results(query);
        } else {
            // This is to prevent lowecased artist names receiving
            // positively biased match scores when filtering by key.
            self.finder.query = query;
            self.selected_index = 0;
            self.offset_y = 0;
        }
    }

    fn sort(&mut self) {
        let query = self.finder.query.clone();
        self.finder.library.fdirs.sort();
        self.update_search_results(query);
    }

    fn clear(&mut self) {
        self.cursor = 0;
        self.offset_y = 0;
        self.update_search_results("".into());
    }

    fn count(&self) -> String {
        format!("{}/{} ", self.finder.matches, self.finder.library.len())
    }

    pub fn selected_dir(&self) -> Option<FuzzyDir> {
        self.finder.library.get(self.selected_index).cloned()
    }

    fn update_finder(&mut self, dir: FuzzyDir) -> EventResult {
        let library = Library::new(&dir.path);
        self.finder.library = library;
        self.clear();
        EventResult::consumed()
    }

    fn load_playlist(&self, next: Playlist) -> EventResult {
        EventResult::with_cb(move |siv| {
            siv.call_on_name(crate::player::ID, |player_view: &mut PlayerView| {
                if player_view.current_dir().path != next.fdir.path {
                    player_view.update_playlist(next.clone(), true);
                }
            })
            .map(|_| _ = Self::remove(siv))
            .unwrap_or_else(|| match Player::try_new(next.clone()) {
                Ok(player) => PlayerView::load(siv, player),
                Err(_) => ErrorView::load(siv, anyhow!("Invalid selection!")),
            });
        })
    }

    fn on_select(&mut self) -> EventResult {
        match self.selected_dir() {
            Some(dir) if dir.contains_subdir => self.update_finder(dir),
            Some(dir) => match Playlist::try_from(dir) {
                Ok(playlist) => self.load_playlist(playlist),
                Err(_) => EventResult::with_cb(|siv| {
                    ErrorView::load(siv, anyhow!("Failed to create playlist!"))
                }),
            },
            None => EventResult::with_cb(|siv| ErrorView::load(siv, anyhow!("Nothing selected!"))),
        }
    }

    fn visible_rows(&self) -> (usize, usize) {
        let last_visible_row = if self.size.y > 2 { self.size.y - 2 } else { 0 };
        let first_visible_row = self.size.y - 1 - min(last_visible_row, self.finder.matches);

        (first_visible_row, last_visible_row)
    }

    fn mouse_select(&mut self, position: XY<usize>) -> EventResult {
        let (first_visible_row, last_visible_row) = self.visible_rows();

        if position.y < first_visible_row || position.y > last_visible_row {
            return EventResult::consumed();
        }
        let next_selected = last_visible_row + self.offset_y - position.y;

        if next_selected == self.selected_index {
            self.on_select()
        } else {
            self.selected_index = next_selected;
            EventResult::consumed()
        }
    }

    fn on_cancel(&self) -> EventResult {
        EventResult::with_cb(|siv| {
            if let None = siv.call_on_name(crate::player::ID, |_: &mut PlayerView| {}) {
                siv.quit();
            } else {
                siv.pop_layer();
            }
        })
    }

    fn spinner_frame(&self) -> Option<&str> {
        self.init_timestamp.and_then(|last_update| {
            let elapsed = last_update.elapsed().as_millis() / 100;
            let index = (elapsed % FRAMES.len() as u128) as usize;
            Some(FRAMES[index])
        })
    }

    pub fn set_init_timestamp(&mut self, ts: Option<Instant>) {
        self.init_timestamp = ts;
    }

    pub fn remove(siv: &mut cursive::Cursive) {
        if siv.find_name::<FinderView>(super::ID).is_some() {
            siv.pop_layer();
        }
    }
}

impl View for FinderView {
    fn layout(&mut self, size: XY<usize>) {
        self.size = size;
    }

    fn draw(&self, p: &Printer) {
        // The size of the screen we can draw on.
        let (w, h) = (p.size.x, p.size.y);

        if h > 3 {
            // The first row of the list.
            let start_row = h - 3;
            // The number of visible rows.
            let visible_rows = min(self.finder.matches - self.offset_y, h - 2);

            for y in 0..visible_rows {
                let index = y + self.offset_y;
                // The items are drawn in ascending order, starting on third row from bottom.
                let row = start_row - y;
                // Only draw items that have matches.
                if self.finder.library[index].match_weight != 0 {
                    // Set the color depending on whether row is currently selected or not.
                    let (primary, highlight) =
                        if row + self.selected_index == start_row + self.offset_y {
                            // Draw the symbol to show the currently selected item.
                            p.with_color(ColorStyles::header_2(), |p| p.print((0, row), ">"));
                            // The colors for the currently selected row.
                            (ColorStyles::hl(), ColorStyles::header_1())
                        } else {
                            // The colors for the not selected row.
                            (ColorStyles::fg(), ColorStyles::hl())
                        };
                    // Draw the item's display name.
                    p.with_color(primary, |p| {
                        p.print((2, row), self.finder.library[index].name.as_str())
                    });
                    // Draw the fuzzy matched indices in a highlighting color.
                    for x in &self.finder.library[index].match_indices {
                        let mut chars = self.finder.library[index].name.chars();
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
            if self.finder.matches > 0 {
                p.with_color(ColorStyles::prompt(), |p| {
                    let rows_per_page = h - 2;
                    let current_page = self.selected_index / rows_per_page;
                    let total_pages = (self.finder.matches + rows_per_page - 1) / rows_per_page - 1;
                    let formatted_page_count = format!(" {}/{}", current_page, total_pages);
                    let start_column = self.size.x - formatted_page_count.chars().count();
                    p.print((start_column, 0), formatted_page_count.as_str());
                });
            }
        }

        if h > 1 {
            // The last row we can draw on.
            let query_row = h - 1;

            // Draw the spinner.
            if let Some(frame) = self.spinner_frame() {
                p.with_color(ColorStyles::info(), |p| {
                    p.print((0, query_row - 1), frame);
                });
            }

            // Draw the match count and some borders.
            p.with_color(ColorStyles::progress(), |p| {
                let lines = min(self.finder.matches / 4, h / 4);
                p.print_vline((w - 1, query_row - 1 - lines), lines, "│");
                p.print_hline((2, query_row - 1), w - 3, "─");
                p.print((2, query_row - 1), &self.count());
            });

            // Draw the text input area that shows the query.
            p.with_color(ColorStyles::hl(), |p| {
                p.print_hline((0, query_row), w, " ");
                p.print((2, query_row), &self.finder.query);
            });

            let c = if self.cursor == self.finder.query.len() {
                "_"
            } else {
                self.finder.query[self.cursor..]
                    .graphemes(true)
                    .next()
                    .expect("should find a char")
            };
            let offset = self.finder.query[..self.cursor].width();
            p.with_effect(Effect::Reverse, |p| {
                p.print((offset + 2, query_row), c);
            });

            // Draw the symbol to show the start of the text input area.
            p.with_color(ColorStyles::prompt(), |p| p.print((0, query_row), ">"));
        }
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char(c) => self.insert(c, true),
            Event::Key(Key::Enter) | Event::CtrlChar('j') => return self.on_select(),
            Event::Key(Key::Esc) => return self.on_cancel(),
            Event::Key(Key::Down) | Event::CtrlChar('n') => self.next(),
            Event::Key(Key::Up) | Event::CtrlChar('p') => self.previous(),
            Event::Key(Key::PageUp) => self.page_up(),
            Event::Key(Key::PageDown) => self.page_down(),
            Event::Key(Key::Backspace) => self.backspace(),
            Event::Key(Key::Del) => self.delete(),
            Event::Key(Key::Left) | Event::CtrlChar('b') => self.cursor_left(),
            Event::Key(Key::Right) | Event::CtrlChar('f') => self.cursor_right(),
            Event::Key(Key::Home) => self.cursor = 0,
            Event::Key(Key::End) => self.cursor = self.finder.query.len(),
            Event::CtrlChar('u') => self.clear(),
            Event::CtrlChar('s') => self.sort(),
            Event::Mouse {
                event, position, ..
            } => match event {
                MouseEvent::Press(MouseButton::Right) => return self.on_cancel(),
                MouseEvent::Press(MouseButton::Left) => return self.mouse_select(position),
                MouseEvent::WheelDown => self.next(),
                MouseEvent::WheelUp => self.previous(),
                _ => (),
            },
            _ => (),
        }
        EventResult::Ignored
    }
}
