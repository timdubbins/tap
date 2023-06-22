use std::cmp;

use cursive::event::{Event, EventResult, Key};
use cursive::theme::{ColorStyle, Effect};
use cursive::traits::View;
use cursive::Printer;

use crate::player::{Player, PlayerStatus};
use crate::theme::*;

pub struct PlayerView {
    player: Player,
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

impl PlayerView {
    pub fn new(player: Player) -> Self {
        Self { player }
    }

    fn player_status(&self) -> (&'static str, ColorStyle) {
        match self.player.status {
            PlayerStatus::Paused => ("|", white()),
            PlayerStatus::Playing => (">", yellow()),
            PlayerStatus::Stopped => (".", red()),
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

        match needs_offset {
            true => match available_y {
                3 => self.player.index,
                4 => match self.player.index == self.player.playlist.len() - 1 {
                    true => self.player.index - 1,
                    false => self.player.index,
                },
                _ => {
                    let diff = self.player.playlist.len() + 2 - available_y;
                    match self.player.index <= diff {
                        true => self.player.index - 1,
                        false => diff,
                    }
                }
            },
            false => 0,
        }
    }
}

impl View for PlayerView {
    fn draw(&self, printer: &Printer) {
        // The file currently loaded in the player.
        let f = &self.player.file;

        // The size of the screen we can draw on.
        let (available_x, available_y) = (printer.size.x, printer.size.y);

        // The last line we can draw on.
        let max_y = available_y - 1;

        // The start of the duration column.
        let dur_x = available_x - 9;

        // The time elapsed since playback started.
        let elapsed = self.player.elapsed().as_secs() as usize;

        // The time remaining until playback completes.
        let remaining = cmp::min(f.duration, f.duration - elapsed);

        // The values needed to draw the progress bar.
        let (length, extra) = ratio(elapsed, f.duration, available_x - 16);

        // Draw the playlist, with rows: 'Track, Title, Duration'.
        if available_y > 2 {
            // The offset needed to make sure we show relevant rows.
            let y_offset = self.y_offset(available_y);

            for (i, f) in self.player.playlist.iter().enumerate() {
                // Skip rows that are not visible.
                if i < y_offset {
                    continue;
                }

                if i == self.player.index {
                    // Draw the active row, including the player status.
                    let (symbol, color) = self.player_status();
                    printer.with_color(color, |printer| {
                        printer.print((3, i + 1 - y_offset), symbol)
                    });
                    printer.with_color(white(), |printer| {
                        printer.print(
                            (6, i + 1 - y_offset),
                            format!("{:02}  {}", f.track, f.title).as_str(),
                        );
                        printer.print(
                            (dur_x, i + 1 - y_offset),
                            mins_and_secs(f.duration).as_str(),
                        );
                    })
                } else if i + 1 - y_offset < max_y {
                    // Draw the inactive rows.
                    printer.with_color(blue(), |printer| {
                        printer.print(
                            (6, i + 1 - y_offset),
                            format!("{:02}  {}", f.track, f.title).as_str(),
                        );
                        printer.print(
                            (dur_x, i + 1 - y_offset),
                            mins_and_secs(f.duration).as_str(),
                        );
                    })
                }

                // The active row has been drawn so we can exit early.
                if available_y == 3 {
                    break;
                }
            }
        }

        // Draw the header: 'Artist, Album, Year'.
        if available_y > 1 {
            printer.with_effect(Effect::Bold, |printer| {
                printer.with_color(green(), |printer| printer.print((2, 0), &f.artist.as_str()));
                printer.with_effect(Effect::Italic, |printer| {
                    printer.with_color(yellow(), |printer| {
                        printer.print((f.x_offset, 0), &self.album_and_year().as_str())
                    })
                })
            })
        }

        // Draw the elapsed and remaining playback times.
        printer.with_color(white(), |printer| {
            printer.print((0, max_y), &mins_and_secs(elapsed));
            printer.print((dur_x, max_y), mins_and_secs(remaining).as_str())
        });

        // Draw the fractional part of the progress bar.
        printer.with_color(magenta().invert(), |printer| {
            printer.with_effect(Effect::Reverse, |printer| {
                printer.print((length + 8, max_y), sub_block(extra));
            });
        });

        // Draw the solid part of the progress bar (preceding the fractional part).
        printer
            .cropped((length + 8, available_y))
            .with_color(magenta(), |printer| {
                printer.print_hline((8, max_y), length, "█");
            });

        // Draw spaces to maintain consistent padding when resizing.
        printer.print((available_x - 2, 0), "  ");
        printer.print((available_x - 2, max_y), "  ");
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Refresh => {
                self.player.poll_sink();
                EventResult::Consumed(None)
            }

            Event::Key(Key::Enter) | Event::Char('g') => {
                if self.player.select_track() {
                    self.player.play_or_pause()
                }
                EventResult::Consumed(None)
            }

            Event::Char('G') => {
                self.player.play_last_track();
                EventResult::Consumed(None)
            }

            Event::Char('p') | Event::Char(' ') => {
                self.player.play_or_pause();
                EventResult::Consumed(None)
            }

            Event::Char('s') | Event::Char('.') => {
                self.player.stop();
                EventResult::Consumed(None)
            }

            Event::Char('j')
            | Event::Char('l')
            | Event::Key(Key::Down)
            | Event::Key(Key::Right) => {
                self.player.next();
                EventResult::Consumed(None)
            }

            Event::Char('k') | Event::Char('h') | Event::Key(Key::Up) | Event::Key(Key::Left) => {
                self.player.prev();
                EventResult::Consumed(None)
            }

            Event::Char('m') => {
                self.player.toggle_mute();
                EventResult::Consumed(None)
            }

            Event::Char('0') => {
                self.player.numbers_pressed.push(0);
                EventResult::Consumed(None)
            }

            Event::Char('1') => {
                self.player.numbers_pressed.push(1);
                EventResult::Consumed(None)
            }

            Event::Char('2') => {
                self.player.numbers_pressed.push(2);
                EventResult::Consumed(None)
            }

            Event::Char('3') => {
                self.player.numbers_pressed.push(3);
                EventResult::Consumed(None)
            }

            Event::Char('4') => {
                self.player.numbers_pressed.push(4);
                EventResult::Consumed(None)
            }

            Event::Char('5') => {
                self.player.numbers_pressed.push(5);
                EventResult::Consumed(None)
            }

            Event::Char('6') => {
                self.player.numbers_pressed.push(6);
                EventResult::Consumed(None)
            }

            Event::Char('7') => {
                self.player.numbers_pressed.push(7);
                EventResult::Consumed(None)
            }

            Event::Char('8') => {
                self.player.numbers_pressed.push(8);
                EventResult::Consumed(None)
            }

            Event::Char('9') => {
                self.player.numbers_pressed.push(9);
                EventResult::Consumed(None)
            }

            _ => EventResult::Consumed(None),
        }
    }
}
