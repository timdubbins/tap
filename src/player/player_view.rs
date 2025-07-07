use std::time::Duration;

use cursive::{
    theme::Style,
    utils::{markup::StyledString, span::SpannedString},
};

use {
    cursive::{
        event::{Event, EventResult, MouseButton, MouseEvent},
        theme::{ColorStyle, Effect},
        traits::View,
        view::Nameable,
        CbSink, Cursive, Printer, XY,
    },
    expiring_bool::ExpiringBool,
};

use crate::{
    config::{
        keybinding::{Action, PLAYER_EVENT_TO_ACTION},
        ColorStyles,
    },
    finder::{FuzzyDir, Library},
    player::{AudioFile, HelpView, PlaybackStatus, Player, Playlist},
};

const SEEK_TIME: Duration = Duration::from_secs(10);

// Drawing constants
const SPACER: &'static str = "  ";
const TICK: u32 = 5;

// A struct representing the view and state of the audio player.
pub struct PlayerView {
    // The `AudioPlayer` instance responsible for handling audio playback.
    player: Player,
    // The time to seek to, in seconds. Set to `Some` when a seek operation has been initiated.
    mouse_seek_time: Option<usize>,
    // Whether the current volume level is displayed in the UI, managed with an `ExpiringBool`.
    showing_volume: ExpiringBool,
    // The list of numbers from last keyboard input.
    number_input: Vec<usize>,
    // Whether or not a double-tap event was registered.
    timed_bool: ExpiringBool,
    // Whether or not the player is being shown.
    is_visible: bool,
    // The elapsed playback time in whole seconds.
    elapsed: usize,
    duration: usize,
    index: usize,

    // The styled strings required to render the header and playlist.
    lines: Vec<SpannedString<Style>>,
    // The vertical offset required to ensure the current track is visible in the playlist.
    offset_y: usize,
    // The dimensions of the view, in cells.
    size: XY<usize>,
    // A sender for scheduling callbacks to be executed by the Cursive root.
    cb_sink: CbSink,
}

impl PlayerView {
    pub fn new(player: Player, cb_sink: CbSink) -> Self {
        Self {
            player,
            cb_sink,
            mouse_seek_time: None,
            offset_y: 0,
            is_visible: true,
            elapsed: 0,
            duration: 0,
            index: 0,
            lines: Vec::new(),
            showing_volume: ExpiringBool::new(false, Duration::from_millis(1500)),
            number_input: vec![],
            timed_bool: ExpiringBool::new(false, Duration::from_millis(500)),
            size: XY::default(),
        }
    }

    pub fn load(siv: &mut Cursive, player: Player) {
        let cb_sink = siv.cb_sink().clone();
        let player_view = PlayerView::new(player, cb_sink).with_name(super::ID);

        siv.set_fps(TICK);

        siv.pop_layer();
        siv.add_layer(player_view);
    }

    pub fn update_playlist(&mut self, next: Playlist, set_playing: bool) {
        let is_stopped = self.player.is_stopped();

        let volume = if self.player.is_muted {
            0.0
        } else {
            (self.player.volume as f32) / 100.0
        };

        self.player.previous = Some(self.player.current.clone());
        self.player.current = next;
        self.player.stop();
        self.player.play();
        self.player.set_volume(volume);

        if !set_playing && is_stopped {
            self.player.stop();
        }

        self.lines.clear();
        self.show();
    }

    pub fn hide(&mut self) {
        self.is_visible = false;

        _ = self.cb_sink.send(Box::new(|siv| {
            siv.set_fps(0);
        }));
    }

    pub fn show(&mut self) {
        self.is_visible = true;

        _ = self.cb_sink.send(Box::new(|siv| {
            siv.set_fps(TICK);
        }));
    }

    fn play_or_pause(&mut self) {
        match self.player.status {
            PlaybackStatus::Paused => self.player.resume(),
            PlaybackStatus::Playing => {
                // self.was_paused.store(true, Ordering::Relaxed);
                self.player.pause()
            }
            PlaybackStatus::Stopped => self.player.play(),
        };
    }

    fn increase_volume(&mut self) {
        self.player.increment_volume();
        self.showing_volume.set();
    }

    fn decrease_volume(&mut self) {
        self.player.decrement_volume();
        self.showing_volume.set();
    }

    fn next(&mut self) {
        if self.player.is_randomized {
            self.random_track_and_album();
        } else if self.player.is_shuffled {
            self.shuffled_track();
        } else {
            self.player.increment_track();
        }
    }

    fn previous(&mut self) {
        if self.player.is_randomized || self.player.is_shuffled {
            self.player
                .previous
                .clone()
                .map(|next| self.update_playlist(next, false));
        } else {
            self.player.decrement_track();
        }
    }

    fn random_track_and_album(&mut self) {
        let mut current = self.player.current.clone();

        _ = self.cb_sink.send(Box::new(|siv| {
            let next = match siv.user_data::<Library>() {
                Some(library) => {
                    let dirs = library.audio_dirs();
                    Playlist::randomized_track(current, &dirs)
                }
                None => {
                    current.set_random_index();
                    current
                }
            };

            siv.call_on_name(super::ID, |pv: &mut PlayerView| {
                pv.update_playlist(next, false);
            });
        }));
    }

    // Selects a random track from the current playlist.
    fn shuffled_track(&mut self) {
        let mut current = self.player.current.clone();
        current.set_random_index();
        self.update_playlist(current, false);
    }

    // Callback to select the previous album.
    pub fn previous_album(siv: &mut Cursive) {
        let updated = siv.call_on_name(super::ID, |player_view: &mut PlayerView| {
            if let Some(mut previous) = player_view.player.previous.clone() {
                previous.index = 0;
                player_view.update_playlist(previous, false);
                true
            } else {
                false
            }
        });

        if updated == Some(true) {
            crate::finder::FinderView::remove_finder_view(siv);
        }
    }

    // Callback to select a random album.
    pub fn random_album(siv: &mut Cursive) {
        let dirs = match siv.user_data::<Library>() {
            Some(library) => library.audio_dirs(),
            None => return,
        };

        siv.call_on_name(super::ID, |player_view: &mut PlayerView| {
            let current = player_view.player.current.clone();
            let next = Playlist::randomized(current, &dirs);
            player_view.update_playlist(next, false);
        })
        .unwrap_or_else(|| {
            Playlist::some_randomized(&dirs)
                .and_then(|next| Player::try_new(next).ok())
                .map(|player| PlayerView::load(siv, player));
        });

        crate::finder::FinderView::remove_finder_view(siv);
    }

    // Play the track selected from keyboard input.
    fn play_track_number(&mut self) {
        if let Some(index) = self.map_input_to_index() {
            self.player.play_index(index);
        } else {
            if self.timed_bool.is_true() {
                self.player.play_index(0);
            } else {
                self.timed_bool.set();
            }
        }
    }

    fn map_input_to_index(&mut self) -> Option<usize> {
        let track = concatenate_digits(&self.number_input) as u32;

        let index = self
            .player
            .current
            .audio_files
            .iter()
            .position(|f| f.track == track);

        self.number_input.clear();

        index
    }

    fn parse_seek_input(&mut self) -> Option<Duration> {
        if self.number_input.is_empty() {
            return None;
        } else {
            let time = concatenate_digits(&self.number_input) as u64;
            self.number_input.clear();

            Some(Duration::new(time, 0))
        }
    }

    // Seeks the playback to the input time in seconds.
    fn seek_to_sec(&mut self) {
        self.parse_seek_input().map(|secs| {
            self.player.seek_to_time(secs);
        });
    }

    // Seeks the playback to the input time in minutes.
    fn seek_to_min(&mut self) {
        self.parse_seek_input().map(|secs| {
            self.player.seek_to_time(secs * 60);
        });
    }

    // Handles the mouse left button press actions.
    fn mouse_button_left(&mut self, offset: XY<usize>, position: XY<usize>) {
        match Area::from(position, offset, self.size) {
            Area::ProgressBar => self.mouse_hold_seek(offset, position),

            Area::Playlist => {
                let index = position.y - offset.y + self.offset_y - 1;

                if index == self.player.current.index {
                    self.play_or_pause();
                } else {
                    self.player.play_index(index);
                }
            }
            Area::Background => _ = self.play_or_pause(),
        }
    }

    // Updates the seek position from mouse input.
    fn mouse_hold_seek(&mut self, offset: XY<usize>, position: XY<usize>) {
        if self.size.x > 16 && position.x > offset.x {
            if self.player.is_stopped() {
                self.player.play();
            }
            self.player.pause();
            let position = (position.x - offset.x).clamp(8, self.size.x - 8) - 8;
            self.mouse_seek_time = Some(position * self.duration() / (self.size.x - 16));
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
        match Area::from(position, offset, self.size) {
            Area::Playlist => match event {
                MouseEvent::WheelUp => self.player.decrement_track(),
                MouseEvent::WheelDown => self.player.increment_track(),
                _ => (),
            },
            _ => match event {
                MouseEvent::WheelUp => self.increase_volume(),
                MouseEvent::WheelDown => self.decrease_volume(),
                _ => (),
            },
        }
    }

    #[inline]
    pub fn current_dir(&self) -> &FuzzyDir {
        &self.player.current.fdir
    }

    #[inline]
    pub fn duration(&self) -> usize {
        self.player.current_file().duration
    }

    // Formats the display for the current playback status.
    #[inline]
    fn playback_status(&self) -> (&'static str, ColorStyle, Effect) {
        match self.player.status {
            PlaybackStatus::Paused => ("|", ColorStyles::hl(), Effect::Simple),
            PlaybackStatus::Playing => (">", ColorStyles::header_2(), Effect::Simple),
            PlaybackStatus::Stopped => (".", ColorStyles::err(), Effect::Simple),
        }
    }

    // Formats the display showing whether the player is muted or randomized.
    #[inline]
    fn playback_opts(&self) -> Option<&'static str> {
        match (
            self.player.is_randomized,
            self.player.is_shuffled,
            self.player.is_muted,
        ) {
            (true, false, true) => Some(" *m"),
            (false, true, true) => Some(" ~m"),
            (true, false, false) => Some("  *"), // is_randomized
            (false, true, false) => Some("  ~"), // is_shuffled
            (false, false, true) => Some("  m"), //is_muted
            _ => None,
        }
    }

    // Formats the player header.
    #[inline]
    fn album_and_year(&self, f: &AudioFile) -> String {
        if let Some(year) = f.year {
            format!("{} ({})", f.album, year)
        } else {
            f.album.to_string()
        }
    }

    // Formats the volume display.
    #[inline]
    fn volume(&self) -> String {
        format!("{}vol: {:>3} %{}", SPACER, self.player.volume, SPACER)
    }

    // Computes the y offset needed to show the results of the fuzzy match.
    #[inline]
    fn update_offset(&self) -> usize {
        let index = self.player.current.index;
        let available_y = self.size.y;
        let required_y = self.player.current.audio_files.len() + 2;
        let offset = required_y.saturating_sub(available_y);

        std::cmp::min(index, offset)
    }

    fn build_static_lines(&mut self) {
        let artist_style = Style::from(ColorStyles::header_1()).combine(Effect::Bold);
        let album_style = Style::from(ColorStyles::header_2())
            .combine(Effect::Bold)
            .combine(Effect::Italic);
        let active_style = ColorStyles::hl();
        let inactive_style = ColorStyles::fg();

        for f in self.player.current.audio_files.iter() {
            let mut track = StyledString::new();
            let mut duration = StyledString::new();
            let mut curr_track = StyledString::new();
            let mut curr_duration = StyledString::new();
            let mut header = StyledString::new();

            let track_str = format!("{:02}{}{}", f.track, SPACER, f.title);
            let duration_str = mins_and_secs(f.duration);

            track.append_styled(track_str.clone(), inactive_style);
            self.lines.push(track);

            duration.append_styled(duration_str.clone(), inactive_style);
            self.lines.push(duration);

            curr_track.append_styled(track_str, active_style);
            self.lines.push(curr_track);

            curr_duration.append_styled(duration_str, active_style);
            self.lines.push(curr_duration);

            header.append_plain(SPACER);
            header.append_styled(f.artist.clone(), artist_style);
            header.append_plain(SPACER);
            header.append_styled(self.album_and_year(&f), album_style);
            self.lines.push(header);
        }
    }
}

impl View for PlayerView {
    fn layout(&mut self, size: XY<usize>) {
        if self.player.is_playing() {
            if self.player.is_randomized {
                if self.player.is_empty() {
                    self.random_track_and_album();
                }
            } else if self.player.is_shuffled {
                if self.player.is_empty() {
                    self.shuffled_track();
                }
            } else {
                self.player.update_on_poll()
            }
        }

        self.elapsed = match self.mouse_seek_time {
            Some(t) if self.player.is_paused() => t,
            _ => self.player.elapsed().as_secs() as usize,
        };

        self.duration = self.player.current_file().duration;
        self.index = self.player.current.index;

        self.size = self.required_size(size);
        self.offset_y = self.update_offset();

        if self.lines.is_empty() {
            self.build_static_lines();
        }
    }

    fn required_size(&mut self, constraint: cursive::Vec2) -> cursive::Vec2 {
        let player_size = self.player.current.xy_size;

        let size = XY {
            x: std::cmp::min(player_size.x, constraint.x),
            y: std::cmp::min(player_size.y, constraint.y),
        };

        size
    }

    fn draw(&self, p: &Printer) {
        if !self.is_visible {
            return;
        }

        let (w, h) = (self.size.x, self.size.y);
        let dur_col = w.saturating_sub(9);

        // HEADER
        if h > 1 {
            // Artist + album + year
            if let Some(line) = self.lines.chunks(5).nth(self.index) {
                if let Some(header) = line.get(4) {
                    p.print_styled((0, 0), header);
                }
            }

            // Volume
            if self.showing_volume.is_true() {
                p.with_color(ColorStyles::prompt(), |p| {
                    p.print((dur_col.saturating_sub(5), 0), &self.volume())
                });
            };
        }

        // PLAYLIST
        for (i, track_lines) in self
            .lines
            .chunks(5)
            .enumerate()
            .skip(self.offset_y)
            .take(h - 2)
        {
            let is_current = i == self.player.current.index;
            let row = 1 + i - self.offset_y;

            let track_line = if is_current {
                &track_lines[2]
            } else {
                &track_lines[0]
            };
            let duration_line = if is_current {
                &track_lines[3]
            } else {
                &track_lines[1]
            };

            // Playback status
            if is_current {
                let (symbol, color, effect) = self.playback_status();
                p.with_color(color, |p| {
                    p.with_effect(effect, |p| p.print((3, row), symbol))
                });
            }

            // Track + title
            p.print_styled((6, row), track_line);

            // Playback option
            if is_current {
                if let Some(option) = self.playback_opts() {
                    p.with_color(ColorStyles::info(), |p| {
                        p.with_effect(Effect::Italic, |p| {
                            p.print((dur_col.saturating_sub(3), row), option)
                        })
                    })
                }
            }

            // Duration
            p.print_styled((dur_col, row), duration_line);
        }

        // FOOTER
        if h > 0 {
            let last_row = h - 1;
            let (length, extra) = ratio(self.elapsed, self.duration, w.saturating_sub(16));
            let remaining = self.duration.saturating_sub(self.elapsed);

            // Elapsed time
            p.with_color(ColorStyles::hl(), |p| {
                p.print((0, last_row), &mins_and_secs(self.elapsed));
            });

            // Progress bar
            p.with_color(ColorStyles::progress(), |p| {
                p.print((length + 8, last_row), sub_block(extra));
            });
            p.cropped((length + 8, h))
                .with_color(ColorStyles::progress(), |p| {
                    p.print_hline((8, last_row), length, "█");
                });

            // Remaining time
            p.with_color(ColorStyles::hl(), |p| {
                p.print((dur_col, last_row), &mins_and_secs(remaining))
            });
        }
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        use Action::*;
        use MouseEvent::*;

        if let Some(action) = PLAYER_EVENT_TO_ACTION.get(&event) {
            match action {
                PlayOrPause => self.play_or_pause(),
                Stop => self.player.stop(),
                Next => self.next(),
                Previous => self.previous(),
                IncreaseVolume => self.increase_volume(),
                DecreaseVolume => self.decrease_volume(),
                ToggleMute => self.player.toggle_mute(),
                ToggleShowingVolume => _ = self.showing_volume.toggle(),
                SeekToMin => self.seek_to_min(),
                SeekToSec => self.seek_to_sec(),
                SeekForward => self.player.seek_forward(SEEK_TIME),
                SeekBackward => self.player.seek_backward(SEEK_TIME),
                ToggleRandomize => self.player.toggle_randomize(),
                ToggleShuffle => self.player.toggle_shuffle(),
                PlayTrackNumber => self.play_track_number(),
                PlayLastTrack => self.player.play_last_track(),
                ShowHelp => return show_help_view(),
                Quit => return quit(),
            }
        } else {
            match event {
                Event::Char(c) if c.is_ascii_digit() => {
                    self.number_input.push(c.to_digit(10).unwrap() as usize);
                }
                Event::Mouse {
                    event,
                    offset,
                    position,
                } => match event {
                    Press(MouseButton::Left) => self.mouse_button_left(offset, position),
                    Press(MouseButton::Right) => self.player.stop(),
                    Release(MouseButton::Left) => self.mouse_release_seek(),
                    Hold(MouseButton::Left) => {
                        if self.mouse_seek_time.is_some() {
                            self.mouse_hold_seek(offset, position);
                        }
                    }
                    WheelUp | WheelDown => self.mouse_wheel(event, offset, position),
                    _ => (),
                },
                _ => (),
            }
        }

        EventResult::Ignored
    }
}

fn quit() -> EventResult {
    EventResult::with_cb(|siv| {
        siv.quit();
    })
}

fn show_help_view() -> EventResult {
    EventResult::with_cb(|siv| {
        HelpView::load(siv);
    })
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
    format!("{}{:02}:{:02}{}", SPACER, secs / 60, secs % 60, SPACER)
}

// Represents different areas of the player.
enum Area {
    ProgressBar,
    Playlist,
    Background,
}

impl Area {
    fn from(position: XY<usize>, offset: XY<usize>, size: XY<usize>) -> Self {
        let translation_y = position.y - offset.y;

        if position.y <= offset.y
            || translation_y > size.y
            || position.x <= offset.x + 1
            || position.x + 2 - offset.x >= size.x
            || size.x <= 16
        {
            return Area::Background;
        }

        if translation_y >= size.y - 2 && translation_y <= size.y {
            return Area::ProgressBar;
        }

        Area::Playlist
    }
}

// Concatenates the single-digit numbers in the input array into a single number.
// For example, given [1, 2, 3], the function returns 123.
// Assumes all elements of the array are between 0 and 9.
fn concatenate_digits(arr: &[usize]) -> usize {
    arr.iter().fold(0, |acc, x| acc * 10 + x)
}
