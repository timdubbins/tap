use std::cmp::min;
use std::collections::VecDeque;
use std::path::PathBuf;

use cursive::event::{Event, EventResult, Key, MouseButton, MouseEvent};
use cursive::theme::{ColorStyle, Effect};
use cursive::traits::View;
use cursive::view::Resizable;
use cursive::{Cursive, Printer};

use crate::app::remove_layers_to_top;
use crate::player::{Player, PlayerStatus, Size};
use crate::theme::*;

pub struct PlayerView {
    player: Player,
}

impl PlayerView {
    pub fn new(player: Player) -> Self {
        Self { player }
    }

    pub fn load((player, size): (Player, Size), siv: &mut Cursive) {
        let path = player.path.to_owned();

        siv.add_layer(
            PlayerView::new(player)
                .full_width()
                .max_width(std::cmp::max(size.0, 53))
                .fixed_height(size.1),
        );

        remove_layers_to_top(siv);

        // Keep a reference to the current and previous player.
        if siv.user_data::<VecDeque<PathBuf>>().is_none() {
            siv.set_user_data(VecDeque::from([
                PathBuf::from(path.as_path()),
                PathBuf::from(path.as_path()),
            ]));
        } else {
            siv.with_user_data(|history: &mut VecDeque<PathBuf>| {
                history.push_back(path);
                history.pop_front();
            });
        }
    }

    fn player_status(&self) -> (&'static str, ColorStyle, Effect) {
        match self.player.is_muted {
            true => ("m", cyan(), Effect::Italic),
            false => match self.player.status {
                PlayerStatus::Paused => ("|", white(), Effect::Simple),
                PlayerStatus::Playing => (">", yellow(), Effect::Simple),
                PlayerStatus::Stopped => (".", red(), Effect::Simple),
            },
        }
    }

    fn album_and_year(&self) -> String {
        if let Some(year) = self.player.file.year {
            return format!("{} ({})", self.player.file.album, year);
        } else {
            return format!("{}", self.player.file.album);
        }
    }

    fn y_offset(&self, available_y: usize) -> usize {
        let needs_offset = self.player.index > 0 && available_y < self.player.playlist.len() + 2;
        let index = self.player.index;

        match needs_offset {
            true => match available_y {
                3 => index,
                4 => match index == self.player.playlist.len() - 1 {
                    true => index - 1,
                    false => index,
                },
                _ => {
                    let diff = self.player.playlist.len() + 2 - available_y;
                    match index <= diff {
                        true => index - 1,
                        false => diff,
                    }
                }
            },
            false => 0,
        }
    }
}

impl View for PlayerView {
    fn draw(&self, p: &Printer) {
        // The file currently loaded in the player.
        let f = &self.player.file;
        // The size of the screen we can draw on.
        let (w, h) = (p.size.x, p.size.y);
        // The last row we can draw on.
        let last_row = h - 1;
        // The start of the duration column.
        let dur_col = w - 9;
        // The time elapsed since playback started.
        let elapsed = self.player.elapsed().as_secs() as usize;

        // TODO - Event::Refresh is not sent when view is not in focus.
        // This means Player::poll_sink is not called when a FuzzyView
        // is loaded and so elapsed grows larger than duration when a
        // track ends and we crash instead of playing the next track.
        // This check prevents the crash but causes playback to pause
        // until the PlayerView comes into focus again. Ideally we want
        // to have the player continue playback as normal. To do this
        // we will need to remove our dependence on Event::Refresh.
        if f.duration < elapsed {
            return;
        }

        // The time remaining until playback completes.
        let remaining = min(f.duration, f.duration - elapsed);
        // The values needed to draw the progress bar.
        let (length, extra) = ratio(elapsed, f.duration, w - 16);

        // Draw the playlist, with rows: 'Track, Title, Duration'.
        if h > 2 {
            // The offset needed to make sure we show relevant rows.
            let y_offset = self.y_offset(h);

            for (i, f) in self.player.playlist.iter().enumerate() {
                // Skip rows that are not visible.
                if i < y_offset {
                    continue;
                }

                if i == self.player.index {
                    // Draw the player status.
                    let (symbol, color, effect) = self.player_status();
                    p.with_color(color, |p| {
                        p.with_effect(effect, |p| p.print((3, i + 1 - y_offset), symbol))
                    });
                    // Draw the active row.
                    p.with_color(white(), |p| {
                        p.print(
                            (6, i + 1 - y_offset),
                            format!("{:02}  {}", f.track, f.title).as_str(),
                        );
                        p.print(
                            (dur_col, i + 1 - y_offset),
                            mins_and_secs(f.duration).as_str(),
                        );
                    })
                } else if i + 1 - y_offset < last_row {
                    // Draw the inactive rows.
                    p.with_color(blue(), |p| {
                        p.print(
                            (6, i + 1 - y_offset),
                            format!("{:02}  {}", f.track, f.title).as_str(),
                        );
                        p.print(
                            (dur_col, i + 1 - y_offset),
                            mins_and_secs(f.duration).as_str(),
                        );
                    })
                }

                // The active row has been drawn so we can exit early.
                if h == 3 {
                    break;
                }
            }
        }

        // Draw the header: 'Artist, Album, Year'.
        if h > 1 {
            p.with_effect(Effect::Bold, |printer| {
                printer.with_color(green(), |printer| printer.print((2, 0), &f.artist.as_str()));
                printer.with_effect(Effect::Italic, |printer| {
                    printer.with_color(yellow(), |printer| {
                        printer.print((f.x_offset, 0), &self.album_and_year().as_str())
                    })
                })
            })
        }

        // Draw the elapsed and remaining playback times.
        p.with_color(white(), |printer| {
            printer.print((0, last_row), &mins_and_secs(elapsed));
            printer.print((dur_col, last_row), mins_and_secs(remaining).as_str())
        });

        // Draw the fractional part of the progress bar.
        p.with_color(magenta().invert(), |printer| {
            printer.with_effect(Effect::Reverse, |printer| {
                printer.print((length + 8, last_row), sub_block(extra));
            });
        });

        // Draw the solid part of the progress bar (preceding the fractional part).
        p.cropped((length + 8, h)).with_color(magenta(), |printer| {
            printer.print_hline((8, last_row), length, "█");
        });

        // Draw spaces to maintain consistent padding when resizing.
        p.print((w - 2, 0), "  ");
        p.print((w - 2, last_row), "  ");
    }

    fn layout(&mut self, _: cursive::Vec2) {
        self.player.poll_sink();
    }

    // Keybindings for the player view.
    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(Key::Enter) | Event::Char('g') => self.player.play_selection(),
            Event::Char('[') | Event::Key(Key::Home) => self.player.play_first_track(),
            Event::Char(']') | Event::Char('e') | Event::Key(Key::End) => {
                self.player.play_last_track()
            }

            Event::Char('p')
            | Event::Char(' ')
            | Event::Mouse {
                event: MouseEvent::Press(MouseButton::Right),
                ..
            } => self.player.play_or_pause(),

            Event::Char('s') | Event::Char('.') => self.player.stop(),

            Event::Char('j')
            | Event::Char('l')
            | Event::Key(Key::Down)
            | Event::Key(Key::Right)
            | Event::Mouse {
                event: MouseEvent::WheelDown,
                ..
            } => self.player.next(),

            Event::Char('k')
            | Event::Char('h')
            | Event::Key(Key::Up)
            | Event::Key(Key::Left)
            | Event::Mouse {
                event: MouseEvent::WheelUp,
                ..
            } => self.player.prev(),

            Event::Char('m') => self.player.toggle_mute(),

            Event::Char('0') => self.player.numbers_pressed.push(0),
            Event::Char('1') => self.player.numbers_pressed.push(1),
            Event::Char('2') => self.player.numbers_pressed.push(2),
            Event::Char('3') => self.player.numbers_pressed.push(3),
            Event::Char('4') => self.player.numbers_pressed.push(4),
            Event::Char('5') => self.player.numbers_pressed.push(5),
            Event::Char('6') => self.player.numbers_pressed.push(6),
            Event::Char('7') => self.player.numbers_pressed.push(7),
            Event::Char('8') => self.player.numbers_pressed.push(8),
            Event::Char('9') => self.player.numbers_pressed.push(9),

            Event::Char('q') => {
                return EventResult::with_cb(move |siv| {
                    siv.quit();
                });
            }

            _ => return EventResult::Ignored,
        }

        EventResult::Consumed(None)
    }
}

fn ratio(value: usize, max: usize, length: usize) -> (usize, usize) {
    let integer = length * value / max;
    let fraction = length * value - max * integer;

    let fraction = fraction * 8 / max;

    (integer, fraction)
}

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

fn mins_and_secs(secs: usize) -> String {
    format!("  {:02}:{:02}  ", secs / 60, secs % 60)
}
