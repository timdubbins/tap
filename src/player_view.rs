use cursive::event::{Event, EventResult};
use cursive::theme::Effect;
use cursive::traits::View;
use cursive::Printer;

use crate::player::{Player, PlayerStatus};

pub struct PlayerView {
    player: Player,
    size: usize,
}

impl PlayerView {
    pub fn new(player: Player, size: usize) -> Self {
        Self { player, size }
    }
}

impl View for PlayerView {
    fn draw(&self, printer: &Printer) {
        let f = &self.player.file;
        let elapsed = self.player.elapsed().as_secs();

        let header = match f.year {
            Some(y) => format!("{} - {} - {}", f.artist, f.album, y),
            None => format!("{} - {}", f.artist, f.album),
        };

        let status = match self.player.status {
            PlayerStatus::Paused => "||",
            PlayerStatus::Playing => ">",
            PlayerStatus::Stopped => ".",
        };

        printer.with_effect(Effect::Underline, |p| {
            p.print((2, 1), &header.as_str());
        });
        for (y, f) in self.player.playlist.iter().enumerate() {
            let line = format!("{:02} - {} - {}", f.track, f.title, f.duration_display);

            if y == self.player.index {
                printer.print((3, y + 2), status);
            }

            printer.print((6, y + 2), &line);
        }

        printer.print(
            (2, self.size - 2),
            &format!("{:02}:{:02}", elapsed / 60, elapsed % 60),
        );
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Refresh => {
                self.player.poll_sink();
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

            _ => EventResult::Consumed(None),
        }
    }
}
