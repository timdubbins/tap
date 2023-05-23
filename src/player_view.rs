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
}

impl View for PlayerView {
    fn draw(&self, printer: &Printer) {
        let f = &self.player.file;
        let (available_x, available_y) = (printer.size.x, printer.size.y);
        let elapsed = self.player.elapsed().as_secs() as usize;
        let remaining = cmp::min(f.duration, f.duration - elapsed);
        let (length, extra) = ratio(elapsed, f.duration, available_x - 16);

        // Draw the header: 'Artist, Album, Year'.
        printer.with_effect(Effect::Bold, |printer| {
            printer.with_color(green(), |printer| printer.print((2, 1), &f.artist.as_str()));
            printer.with_effect(Effect::Italic, |printer| {
                printer.with_color(yellow(), |printer| {
                    printer.print((f.offset, 1), &self.album_and_year().as_str())
                })
            })
        });

        // Draw the playlist, with rows: 'Track, Title, Duration'.
        for (i, f) in self.player.playlist.iter().enumerate() {
            if i == self.player.index {
                // Draw the active row with player status and highlighting.
                let (symbol, color) = self.player_status();
                printer.with_color(color, |printer| printer.print((3, i + 2), symbol));
                printer.with_color(white(), |printer| {
                    printer.print((6, i + 2), format!("{:02}  {}", f.track, f.title).as_str());
                    printer.print((available_x - 9, i + 2), mins_and_secs(f.duration).as_str());
                });
            } else {
                // Draw the inactive rows.
                printer.with_color(blue(), |printer| {
                    printer.print((6, i + 2), format!("{:02}  {}", f.track, f.title).as_str());
                    printer.print((available_x - 9, i + 2), mins_and_secs(f.duration).as_str());
                });
            }

            // Draw the elapsed and remaining playback times, in mins and secs.
            printer.with_color(white(), |printer| {
                printer.print((0, available_y - 2), &mins_and_secs(elapsed));
                printer.print(
                    (available_x - 9, available_y - 2),
                    mins_and_secs(remaining).as_str(),
                )
            });

            // Draw the fractional part of the progress bar.
            printer.with_color(magenta().invert(), |printer| {
                printer.with_effect(Effect::Reverse, |printer| {
                    printer.print((length + 8, available_y - 2), sub_block(extra));
                });
            });

            // Draw the rest of the progress bar (preceding the fractional part).
            printer
                .cropped((length + 8, available_y))
                .with_color(magenta(), |printer| {
                    printer.print_hline((8, available_y - 2), length, "█");
                });

            // Crop the RHS of the header and progress bar by drawing spaces.
            // This maintains consistent padding when resizing.
            printer.print((available_x - 2, 1), "  ");
            printer.print((available_x - 2, available_y - 2), "  ");
        }
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
