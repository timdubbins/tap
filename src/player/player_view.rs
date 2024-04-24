use std::time::Duration;

use cursive::{
    event::{Event, EventResult, Key, MouseButton, MouseEvent},
    reexports::crossbeam_channel::Sender,
    theme::{ColorStyle, Effect},
    traits::View,
    view::Resizable,
    Cursive, Printer, XY,
};
use expiring_bool::ExpiringBool;

use crate::config::{args, theme};
use crate::fuzzy::{self, FuzzyView};
use crate::session_data::SessionData;
use crate::utils::{self, InnerType};

use super::{AudioFile, KeysView, Player, PlayerBuilder, PlayerStatus};

pub struct PlayerView {
    // The currently loaded player.
    player: Player,
    // The time to seek to, in seconds. `Some` when seeking has been initiated.
    mouse_seek_time: Option<usize>,
    // The vertical offset required to show relevant playlist rows.
    offset: usize,
    // Whether or not the current volume is displayed.
    showing_volume: ExpiringBool,
    // Callback to access the cursive root. `None` if standalone player.
    cb: Option<Sender<Box<dyn FnOnce(&mut Cursive) + Send>>>,
    // The size of the view.
    size: XY<usize>,
}

impl PlayerView {
    pub fn new(
        player: Player,
        showing_volume: bool,
        cb: Option<Sender<Box<dyn FnOnce(&mut Cursive) + Send>>>,
    ) -> Self {
        Self {
            player,
            cb,
            mouse_seek_time: None,
            offset: 0,
            showing_volume: ExpiringBool::new(showing_volume, Duration::from_millis(1500)),
            size: XY { x: 0, y: 0 },
        }
    }

    pub fn load((player, showing_volume, size): (Player, bool, XY<usize>), siv: &mut Cursive) {
        let cb = match siv.user_data::<InnerType<SessionData>>() {
            Some(_) => Some(siv.cb_sink().clone()),
            None => None,
        };

        siv.add_layer(
            PlayerView::new(player, showing_volume, cb)
                .full_width()
                .max_width(size.x)
                .fixed_height(size.y),
        );

        remove_layers_to_top(siv);
    }

    // Draw methods

    // Formats the display for the current playback status.
    fn player_status(&self) -> (&'static str, ColorStyle, Effect) {
        match self.player.status {
            PlayerStatus::Paused => ("|", theme::hl(), Effect::Simple),
            PlayerStatus::Playing => (">", theme::header2(), Effect::Simple),
            PlayerStatus::Stopped => (".", theme::err(), Effect::Simple),
        }
    }

    // Formats the display showing whether the player is muted or randomized.
    fn player_info(&self) -> &'static str {
        match (self.player.is_randomized, self.player.is_muted) {
            (true, true) => " *m",
            (true, false) => "  *",
            (false, true) => "  m",
            (false, false) => unreachable!(),
        }
    }

    // Formats the player header.
    fn album_and_year(&self, f: &AudioFile) -> String {
        if let Some(year) = f.year {
            return format!("{} ({})", f.album, year);
        } else {
            return format!("{}", f.album);
        }
    }

    // Formats the volume display.
    fn volume(&self, w: usize) -> String {
        match w > 14 {
            true => format!("  vol: {:>3} %  ", self.player.volume),
            false => format!("  {:>3} %  ", self.player.volume),
        }
    }

    // The elapsed playback time to display. When seeking with the mouse we use the
    // elapsed time had the seeking process completed.
    fn elapsed(&self) -> usize {
        if self.mouse_seek_time.is_some() && self.player.status == PlayerStatus::Paused {
            self.mouse_seek_time.unwrap()
        } else {
            self.player.elapsed().as_secs() as usize
        }
    }

    // Computes the y offset needed to show the results of the fuzzy match.
    #[inline]
    fn update_offset(&self) -> usize {
        let index = self.player.index;
        let length = self.player.playlist.len();
        let available_y = self.size.y;
        let required_y = length + 2;

        if index == 0 || available_y >= required_y {
            return 0;
        }

        let offset = required_y - available_y;
        if index <= offset {
            index
        } else {
            offset
        }
    }

    // Event methods

    // Loads the next random track.
    fn random_track(&mut self) {
        match &self.cb {
            Some(cb) => {
                cb.send(Box::new(move |siv| {
                    if let Ok(player) = PlayerBuilder::RandomTrack.from(None, siv) {
                        PlayerView::load(player, siv);
                    }
                }))
                .unwrap_or_default();
            }
            None => self.player.next_random(),
        }
    }

    // Loads the previous random track.
    fn previous_random(&mut self) {
        match &self.cb {
            Some(cb) => {
                cb.send(Box::new(move |siv| {
                    if let Ok(player) = PlayerBuilder::PreviousTrack.from(None, siv) {
                        PlayerView::load(player, siv);
                    }
                }))
                .unwrap_or_default();
            }
            None => self.player.previous_random(),
        }
    }

    // Sets the current volume. If the volume is not being shown
    // it will be displayed temporarily by setting `showing_volume` true.
    fn set_volume(&mut self, volume: u8) -> EventResult {
        self.showing_volume.set();

        if self.cb.is_some() {
            EventResult::with_cb(move |siv| {
                siv.with_user_data(|(opts, _, _): &mut InnerType<SessionData>| {
                    opts.1 = volume;
                });
            })
        } else {
            EventResult::Consumed(None)
        }
    }

    // Updates user data with the current status.
    fn set_status(&mut self, status: u8) -> EventResult {
        if self.cb.is_some() {
            EventResult::with_cb(move |siv| {
                siv.with_user_data(|(opts, _, _): &mut InnerType<SessionData>| {
                    opts.0 = status;
                });
            })
        } else {
            EventResult::Consumed(None)
        }
    }

    // Toggles the track order between in-order and random.
    fn toggle_randomization(&mut self) -> EventResult {
        if self.player.toggle_randomization() {
            let curr_index = self.player.index;
            if self.cb.is_some() {
                return EventResult::with_cb(move |siv| {
                    siv.with_user_data(|(_, _, queue): &mut InnerType<SessionData>| {
                        if let Some((_, index)) = queue.get_mut(1) {
                            *index = curr_index;
                        }
                    });
                });
            } else if self.player.playlist.len() > 1 {
                return EventResult::with_cb(move |siv| siv.set_user_data(curr_index));
            }
        }
        EventResult::Consumed(None)
    }

    // Loads a fuzzy view for the parent of the current audio file.
    fn parent(&self) -> EventResult {
        let mut parent = self.player.path().to_owned();
        let root = args::search_root();

        if parent != root {
            parent.pop();
            if parent != root {
                parent.pop();
                return EventResult::with_cb(move |siv| {
                    let items = fuzzy::create_items(&parent).expect("should always exist");
                    FuzzyView::load(items, None, siv)
                });
            }
        }
        EventResult::Consumed(None)
    }

    // Toggles whether the player is muted and updates user data.
    fn toggle_mute(&mut self) -> EventResult {
        let is_muted = self.player.toggle_mute();
        if self.cb.is_some() {
            EventResult::with_cb(move |siv| {
                siv.with_user_data(|(opts, _, _): &mut InnerType<SessionData>| {
                    opts.2 = is_muted;
                });
            })
        } else {
            EventResult::Consumed(None)
        }
    }

    // Toggles whether or not the volume is displayed and updates user data.
    fn toggle_volume_display(&mut self) -> EventResult {
        let showing_volume = self.showing_volume.toggle();
        if self.cb.is_some() {
            EventResult::with_cb(move |siv| {
                siv.with_user_data(|(opts, _, _): &mut InnerType<SessionData>| {
                    opts.3 = showing_volume;
                });
            })
        } else {
            EventResult::Consumed(None)
        }
    }

    // Loads the next track in the queue.
    fn next(&mut self) {
        if self.player.is_randomized {
            self.random_track();
        } else {
            self.player.next();
        }
    }

    // Loads the previous track in the queue.
    fn previous(&mut self) {
        if self.player.is_randomized {
            self.previous_random();
        } else {
            self.player.previous()
        }
    }

    // Opens the parent of the current audio file in the
    // preferred file manager.
    fn open_file_manager(&self) {
        let path = self.player.path().to_owned();
        _ = utils::open_file_manager(path);
    }

    // Increments the volume and updates user data.
    fn increase_volume(&mut self) -> EventResult {
        let volume = self.player.increase_volume();
        return self.set_volume(volume);
    }

    // Decrements the volume and updates user data.
    fn decrease_volume(&mut self) -> EventResult {
        let volume = self.player.decrease_volume();
        return self.set_volume(volume);
    }

    // Stops the player and updates user data.
    fn stop(&mut self) -> EventResult {
        let status = self.player.stop();
        return self.set_status(status);
    }

    // Plays or pauses the player and updates user data.
    fn play_or_pause(&mut self) -> EventResult {
        let status = self.player.play_or_pause();
        return self.set_status(status);
    }

    // Handles the mouse left button press actions.
    fn mouse_button_left(&mut self, offset: XY<usize>, position: XY<usize>) {
        // Whether or not the mouse cursor is outside the area containing
        // the playlist and the progress bar.
        let outside_area = position.y <= offset.y
            || position.y - offset.y > self.size.y
            || position.x <= offset.x + 1
            || position.x + 2 - offset.x >= self.size.x;

        if outside_area {
            self.play_or_pause();
            return;
        }

        // The y position of the mouse cursor relative to the view.
        let translation_y = position.y - offset.y;

        // Initiate seeking if the mouse cursor is over progress bar or line below.
        if translation_y == self.size.y || translation_y + 1 == self.size.y {
            if self.size.x > 16 {
                self.mouse_hold_seek(offset, position);
            } else {
                self.player.play_or_pause();
            }
            return;
        }

        // Select the track under the mouse cursor.
        let index = translation_y + self.offset - 1;
        if index == self.player.index {
            self.player.play_or_pause();
        } else if index < self.player.playlist.len() {
            self.player.play_mouse_selected(index);
        }
    }

    // Updates the seek position from mouse input.
    fn mouse_hold_seek(&mut self, offset: XY<usize>, position: XY<usize>) {
        if self.size.x > 16 && position.x > offset.x {
            if self.player.status == PlayerStatus::Stopped {
                self.player.play();
            }
            self.player.pause();
            let duration = self.player.file().duration;
            let mouse_seek_pos = utils::clamp(position.x - offset.x, 8, self.size.x - 8) - 8;
            self.mouse_seek_time = Some(mouse_seek_pos * duration / (self.size.x - 16));
        }
    }

    // Performs the seek operation from mouse input.
    fn mouse_release_seek(&mut self) {
        if let Some(secs) = self.mouse_seek_time {
            let seek_time = Duration::new(secs as u64, 0);
            self.player.seek_to_time(seek_time);
        }
        self.mouse_seek_time = None;
    }

    // Handles the mouse wheel (scrolling) actions.
    fn mouse_wheel(&mut self, event: MouseEvent, offset: XY<usize>, position: XY<usize>) {
        // Whether or not the mouse cursor is outside the area containing
        // the playlist.
        let outside_playlist = position.y <= offset.y
            || position.y + 1 - offset.y >= self.size.y
            || position.x <= offset.x + 1
            || position.x + 2 - offset.x >= self.size.x;

        if event == MouseEvent::WheelUp {
            if outside_playlist {
                self.increase_volume();
            } else {
                self.previous();
            }
        } else if event == MouseEvent::WheelDown {
            if outside_playlist {
                self.decrease_volume();
            } else {
                if self.player.index != self.player.playlist.len() - 1 {
                    self.next();
                }
            }
        }
    }
}

impl View for PlayerView {
    fn layout(&mut self, size: cursive::Vec2) {
        self.player.poll();
        if self.player.is_randomized && self.player.next_track_queued {
            self.random_track();
        }
        self.size = size;
        self.offset = self.update_offset();
    }

    fn draw(&self, p: &Printer) {
        // The size of the screen we can draw on.
        let (w, h) = (p.size.x, p.size.y);
        // The file currently loaded in the player.
        let f = self.player.file();
        // The start of the duration column.
        let column = if w > 9 { w - 9 } else { 0 };
        // The length of the progress bar.
        let length = if w > 16 { w - 16 } else { 0 };
        // The time elapsed since playback started.
        let elapsed = self.elapsed();
        // The values needed to draw the progress bar.
        let (length, extra) = ratio(elapsed, f.duration, length);

        // Draw the playlist, with rows: 'Track, Title, Duration'.
        if h > 2 {
            for (i, f) in self.player.playlist.iter().enumerate() {
                // Skip rows that are not visible.
                if i < self.offset {
                    continue;
                }

                let row = i + 1 - self.offset;

                if i == self.player.index {
                    // Draw the player status.
                    let (symbol, color, effect) = self.player_status();
                    p.with_color(color, |p| {
                        p.with_effect(effect, |p| p.print((3, row), symbol))
                    });
                    // Draw the active row.
                    p.with_color(theme::hl(), |p| {
                        p.print((6, row), format!("{:02}  {}", f.track, f.title).as_str());
                        if column > 11 && (self.player.is_randomized || self.player.is_muted) {
                            // Draw the player options.
                            p.with_color(theme::info(), |p| {
                                p.with_effect(Effect::Italic, |p| {
                                    p.print((column - 3, row), self.player_info())
                                })
                            })
                        }
                        p.print((column, row), mins_and_secs(f.duration).as_str());
                    })
                } else if i + 2 - self.offset < h {
                    // Draw the inactive rows.
                    p.with_color(theme::fg(), |p| {
                        p.print((6, row), format!("{:02}  {}", f.track, f.title).as_str());
                        p.print((column, row), mins_and_secs(f.duration).as_str());
                    })
                }

                // The active row has been drawn so we can exit early.
                if h == 3 {
                    break;
                }
            }
        }

        if h > 1 {
            // Draw the header: 'Artist, Album, Year'.
            p.with_effect(Effect::Bold, |p| {
                p.with_color(theme::header1(), |p| p.print((2, 0), &f.artist.as_str()));
                p.with_effect(Effect::Italic, |p| {
                    p.with_color(theme::header2(), |p| {
                        p.print((f.artist.len() + 4, 0), &self.album_and_year(f).as_str())
                    })
                })
            });

            if self.showing_volume.is_true() {
                let column = if w > 14 { column - 5 } else { column };
                p.with_color(theme::prompt(), |p| {
                    p.print((column, 0), &self.volume(w).as_str())
                });
            };
        }

        if h > 0 {
            // The last row we can draw on.
            let last_row = h - 1;

            // Draw the elapsed and remaining playback times.
            p.with_color(theme::hl(), |p| {
                let remaining = if elapsed > f.duration {
                    0
                } else {
                    f.duration - elapsed
                };
                p.print((0, last_row), &mins_and_secs(elapsed));
                p.print((column, last_row), mins_and_secs(remaining).as_str())
            });

            // Draw the fractional part of the progress bar.
            p.with_color(theme::progress(), |p| {
                p.print((length + 8, last_row), sub_block(extra));
            });

            // Draw the solid part of the progress bar (preceding the fractional part).
            p.cropped((length + 8, h))
                .with_color(theme::progress(), |p| {
                    p.print_hline((8, last_row), length, "█");
                });

            // Draw spaces to maintain consistent padding when resizing.
            p.print((w - 2, 0), "  ");
            p.print((w - 2, last_row), "  ");
        }
    }

    // Keybindings for the player view.
    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('h' | ' ') | Event::Key(Key::Left) => return self.play_or_pause(),
            Event::Char('j') | Event::Key(Key::Down) => self.next(),
            Event::Char('k') | Event::Key(Key::Up) => self.previous(),
            Event::Char('l') | Event::Key(Key::Enter | Key::Right) => return self.stop(),

            Event::Char(']') => return self.increase_volume(),
            Event::Char('[') => return self.decrease_volume(),
            Event::Char('v') => return self.toggle_volume_display(),
            Event::Char('m') => return self.toggle_mute(),

            Event::Char('\'') => self.player.seek_to_min(),
            Event::Char('"') => self.player.seek_to_sec(),
            Event::Char('.') => self.player.step_forward(),
            Event::Char(',') => self.player.step_backward(),

            Event::Char('*' | 'r') => return self.toggle_randomization(),
            Event::Char('g') => self.player.play_key_selection(),
            Event::CtrlChar('g') => self.player.play_last_track(),

            Event::Char('0') => self.player.num_keys.push(0),
            Event::Char('1') => self.player.num_keys.push(1),
            Event::Char('2') => self.player.num_keys.push(2),
            Event::Char('3') => self.player.num_keys.push(3),
            Event::Char('4') => self.player.num_keys.push(4),
            Event::Char('5') => self.player.num_keys.push(5),
            Event::Char('6') => self.player.num_keys.push(6),
            Event::Char('7') => self.player.num_keys.push(7),
            Event::Char('8') => self.player.num_keys.push(8),
            Event::Char('9') => self.player.num_keys.push(9),

            Event::CtrlChar('p') => return self.parent(),
            Event::CtrlChar('o') => self.open_file_manager(),
            Event::Char('?') => return load_keys_view(),
            Event::Char('q') => return quit(),

            // TODO: scroll to adjust vertical offset, not select track.
            // FIXME: mouse stop, mouse play, mouse select -> playback is
            // stopped but should be playing.
            Event::Mouse {
                event,
                offset,
                position,
            } => match event {
                MouseEvent::Press(MouseButton::Left) => self.mouse_button_left(offset, position),
                MouseEvent::Press(MouseButton::Right) => return self.stop(),
                MouseEvent::Release(MouseButton::Left) => self.mouse_release_seek(),
                MouseEvent::Hold(MouseButton::Left) => {
                    if self.mouse_seek_time.is_some() {
                        self.mouse_hold_seek(offset, position);
                    }
                }
                MouseEvent::WheelUp | MouseEvent::WheelDown => {
                    self.mouse_wheel(event, offset, position)
                }
                _ => (),
            },
            _ => (),
        }
        EventResult::Consumed(None)
    }
}

// Callback to select the previous album.
pub fn previous_album(_: &Event) -> Option<EventResult> {
    Some(EventResult::with_cb(|siv| {
        if let Ok(player) = PlayerBuilder::PreviousAlbum.from(None, siv) {
            PlayerView::load(player, siv);
        }
    }))
}

// Callback to select a random album.
pub fn random_album(_: &Event) -> Option<EventResult> {
    Some(EventResult::with_cb(|siv| {
        if let Ok(player) = PlayerBuilder::RandomAlbum.from(None, siv) {
            PlayerView::load(player, siv);
        }
    }))
}

// Quit the app.
fn quit() -> EventResult {
    return EventResult::with_cb(|siv| {
        siv.quit();
    });
}

// Shows the keys_view popup.
fn load_keys_view() -> EventResult {
    return EventResult::with_cb(|siv| {
        KeysView::load(siv);
    });
}

// Computes the values required to draw the progress bar.
fn ratio(value: usize, max: usize, length: usize) -> (usize, usize) {
    if max == 0 {
        return (0, 0);
    }

    let integer = length * value / max;
    let fraction = length * value - max * integer;

    (integer, fraction * 8 / max)
}

// The characters needed to draw the fractional part of the progress bar.
fn sub_block(extra: usize) -> &'static str {
    match extra {
        0 => " ",
        1 => "▏",
        2 => "▎",
        3 => "▍",
        4 => "▌",
        5 => "▋",
        6 => "▊",
        7 => "▉",
        _ => "█",
    }
}

// Formats the playback time.
fn mins_and_secs(secs: usize) -> String {
    format!("  {:02}:{:02}  ", secs / 60, secs % 60)
}

// Remove all layers from the view stack except the top layer.
fn remove_layers_to_top(siv: &mut Cursive) {
    while siv.screen().len() > 1 {
        siv.screen_mut()
            .remove_layer(cursive::views::LayerPosition::FromBack(0));
    }
}
