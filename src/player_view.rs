use std::cmp;

use cursive::event::{Event, EventResult, Key};
use cursive::theme::{BaseColor::*, ColorStyle, Effect};
use cursive::traits::View;
use cursive::Printer;

use crate::audio_file::AudioFile;
use crate::player::{Player, PlayerStatus};

pub struct PlayerView {
    player: Player,
    cs: ColorStyle,
    cs_inverted: ColorStyle,
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
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn track_title_duration(file: &AudioFile) -> String {
    format!(
        "{:02} - {} - {}",
        file.track,
        file.title,
        mins_and_secs(file.duration)
    )
}

impl PlayerView {
    pub fn new(player: Player) -> Self {
        Self {
            player,
            cs: ColorStyle::new(Black, Black.light()),
            cs_inverted: ColorStyle::new(Black.light(), Black),
        }
    }

    fn status_symbol(&self) -> &'static str {
        match self.player.status {
            PlayerStatus::Paused => "||",
            PlayerStatus::Playing => ">",
            PlayerStatus::Stopped => ".",
        }
    }

    fn artist_album_year(&self) -> String {
        let f = &self.player.file;

        match f.year {
            Some(y) => format!("{} - {} - {}", f.artist, f.album, y),
            None => format!("{} - {}", f.artist, f.album),
        }
    }
}

impl View for PlayerView {
    fn draw(&self, printer: &Printer) {
        let duration = self.player.file.duration;
        let elapsed = self.player.elapsed().as_secs() as usize;
        let remaining = cmp::min(duration, duration - elapsed);
        let (length, extra) = ratio(elapsed, duration, printer.size.x - 16);

        printer.with_effect(Effect::Underline, |p| {
            p.print((2, 1), &self.artist_album_year().as_str());
        });

        for (i, f) in self.player.playlist.iter().enumerate() {
            if i == self.player.index {
                printer.print((3, i + 2), self.status_symbol());
            }

            printer.print((6, i + 2), track_title_duration(f).as_str());
        }

        printer.print((2, printer.size.y - 2), &mins_and_secs(elapsed));

        printer.print(
            (printer.size.x - 7, printer.size.y - 2),
            &mins_and_secs(remaining),
        );

        printer.with_color(self.cs, |printer| {
            printer.with_effect(Effect::Reverse, |printer| {
                printer.print((length + 8, printer.size.y - 2), sub_block(extra));
            });
        });

        let printer = &printer.cropped((length + 8, printer.size.y));
        printer.with_color(self.cs_inverted, |printer| {
            printer.print_hline((8, printer.size.y - 2), length, "█");
        });
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Refresh => {
                self.player.poll_sink();
                EventResult::Consumed(None)
            }

            Event::Key(Key::Enter) => {
                if self.player.select_track() {
                    self.player.play_or_pause()
                }
                EventResult::Consumed(None)
            }

            Event::Char('p') => {
                self.player.play_or_pause();
                EventResult::Consumed(None)
            }

            Event::Char('s') => {
                self.player.stop();
                EventResult::Consumed(None)
            }

            Event::Char('j') => {
                self.player.next();
                EventResult::Consumed(None)
            }

            Event::Char('k') => {
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
