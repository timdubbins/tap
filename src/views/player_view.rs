use std::cmp::min;
use std::collections::VecDeque;
use std::path::PathBuf;

use cursive::event::{Event, EventResult, Key, MouseButton, MouseEvent};
use cursive::reexports::crossbeam_channel::Sender;
use cursive::theme::{ColorStyle, Effect};
use cursive::traits::View;
use cursive::view::Resizable;
use cursive::{Cursive, Printer, XY};

use crate::player::{Player, PlayerOpts, PlayerStatus};
use crate::theme::*;
use crate::utils::random;
use crate::views::KeysView;

type UserData = ((u8, u8, bool), Vec<PathBuf>, VecDeque<(PathBuf, usize)>);

pub struct PlayerView {
    // The currently loaded player.
    player: Player,
    // The last track index selected by mouse input, if any.
    selected: Option<usize>,
    // The vertical offset required to show relevant playlist rows.
    offset: usize,
    // Callback to access the cursive root. `None` if standalone player.
    cb: Option<Sender<Box<dyn FnOnce(&mut Cursive) + Send>>>,
    // The size of the view.
    size: XY<usize>,
}

impl PlayerView {
    pub fn new(player: Player, cb: Option<Sender<Box<dyn FnOnce(&mut Cursive) + Send>>>) -> Self {
        Self {
            player,
            cb,
            selected: None,
            offset: 0,
            size: XY { x: 0, y: 0 },
        }
    }

    pub fn fuzzy((player, size): (Player, XY<usize>), siv: &mut Cursive) {
        let path = player.path.to_owned();
        let mut player = player;
        let (opts, _, _) = siv.user_data::<UserData>().expect("set on init");

        player.volume = opts.1;
        player.init_volume();

        PlayerView::load((player, size), siv);

        siv.with_user_data(|(_, _, queue): &mut UserData| {
            if queue.len() == 1 {
                queue.push_front((path.to_owned(), 0));
                queue.push_front((path, 0));
            } else {
                queue.pop_front();
                queue.insert(1, (path, 0));
            }
        });
    }

    pub fn random(random_track: bool, siv: &mut Cursive) {
        let (opts, _, queue) = siv.user_data::<UserData>().expect("set on init");

        let opts: PlayerOpts = (*opts).into();
        let (path, index) = queue.back().expect("should always exist").to_owned();

        let (mut player, size) = Player::new(&path).expect("should always be valid");
        let length = player.playlist.len();

        if random_track {
            player.index = index;
            player.file = player.playlist[index].to_owned();
            player.is_randomized = true;
        }

        player.status = opts.status;
        player.volume = opts.volume;
        player.is_muted = opts.is_muted;
        player.init_volume();
        player.set_playback();

        PlayerView::load((player, size), siv);

        siv.with_user_data(|(_, paths, queue): &mut UserData| {
            if queue.len() == 1 {
                let first = queue.front().expect("should always exist").to_owned();
                queue.push_back(first);
            } else {
                queue.pop_front();
            }

            let (path, index) = match Player::randomized(&paths) {
                Some(res) => res,
                None => (path, random(0..length)),
            };

            queue.push_back((path, index));
        });
    }

    pub fn previous(random_track: bool, siv: &mut Cursive) {
        let (opts, _, queue) = siv.user_data::<UserData>().expect("set on init");

        if queue.len() == 1 {
            return;
        }

        let opts: PlayerOpts = (*opts).into();
        let (path, index) = queue.front().expect("should always exist").to_owned();

        let (mut player, size) = Player::new(&path).expect("should always be valid");

        if random_track {
            player.index = index;
            player.file = player.playlist[index].to_owned();
            player.is_randomized = true;
        }

        player.status = opts.status;
        player.volume = opts.volume;
        player.is_muted = opts.is_muted;
        player.init_volume();
        player.set_playback();

        PlayerView::load((player, size), siv);

        siv.with_user_data(|(_, _, queue): &mut UserData| {
            queue.swap(0, 1);
        });
    }

    pub fn load((player, size): (Player, XY<usize>), siv: &mut Cursive) {
        let cb = match siv.user_data::<UserData>() {
            Some(_) => Some(siv.cb_sink().clone()),
            None => None,
        };

        siv.add_layer(
            PlayerView::new(player, cb)
                .full_width()
                .max_width(size.x)
                .fixed_height(size.y),
        );

        remove_layers_to_top(siv);
    }

    fn player_status(&self) -> (&'static str, ColorStyle, Effect) {
        match self.player.status {
            PlayerStatus::Paused => ("|", white(), Effect::Simple),
            PlayerStatus::Playing => (">", yellow(), Effect::Simple),
            PlayerStatus::Stopped => (".", red(), Effect::Simple),
        }
    }

    fn player_opts(&self) -> &'static str {
        match (self.player.is_randomized, self.player.is_muted) {
            (true, true) => " *m",
            (true, false) => "  *",
            (false, true) => "  m",
            (false, false) => unreachable!(),
        }
    }

    fn album_and_year(&self) -> String {
        if let Some(year) = self.player.file.year {
            return format!("{} ({})", self.player.file.album, year);
        } else {
            return format!("{}", self.player.file.album);
        }
    }

    fn volume(&self, w: usize) -> String {
        match w > 14 {
            true => format!("  vol: {:>3} %  ", self.player.volume),
            false => format!("  {:>3} %  ", self.player.volume),
        }
    }

    fn update_offset(&self) -> usize {
        let available_y = self.size.y;
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

    fn mouse_select(&mut self, m_off_y: usize, event: Event) -> EventResult {
        let m_pos_y = event.mouse_position().unwrap_or_default().y;

        // Restrict values to visible rows of the playlist.
        if m_pos_y <= m_off_y || m_pos_y >= m_off_y + self.size.y - 2 {
            return EventResult::Consumed(None);
        }

        // The mouse selected track index.
        let selected = self.offset + m_pos_y - m_off_y - 1;

        if selected == self.player.index {
            self.player.play_or_pause();
        } else if Some(selected) == self.selected {
            self.player.select_track_index(selected);
        } else {
            self.selected = Some(selected);
        }

        EventResult::Consumed(None)
    }

    fn random_track(&mut self) {
        match &self.cb {
            Some(cb) => {
                cb.send(Box::new(move |siv| {
                    PlayerView::random(true, siv);
                }))
                .unwrap_or_default();
            }
            None => self.player.next_random(),
        }
    }

    fn previous_random(&mut self) {
        match &self.cb {
            Some(cb) => {
                cb.send(Box::new(move |siv| {
                    PlayerView::previous(true, siv);
                }))
                .unwrap_or_default();
            }
            None => self.player.previous_random(),
        }
    }
}

impl View for PlayerView {
    fn draw(&self, p: &Printer) {
        // The size of the screen we can draw on.
        let (w, h) = (p.size.x, p.size.y);
        // The file currently loaded in the player.
        let f = &self.player.file;
        // The start of the duration column.
        let column = if w > 9 { w - 9 } else { 0 };
        // The length of the progress bar.
        let length = if w > 16 { w - 16 } else { 0 };
        // The time elapsed since playback started.
        let elapsed = self.player.elapsed().as_secs() as usize;
        // The values needed to draw the progress bar.
        let (length, extra) = ratio(elapsed, f.duration, length);

        // This is to guard against a potential division by zero.
        if f.duration < elapsed {
            return;
        }

        // Draw the playlist, with rows: 'Track, Title, Duration'.
        if h > 2 {
            for (i, f) in self.player.playlist.iter().enumerate() {
                // Skip rows that are not visible.
                if i < self.offset {
                    continue;
                }

                if i == self.player.index {
                    // Draw the player status.
                    let (symbol, color, effect) = self.player_status();
                    p.with_color(color, |p| {
                        p.with_effect(effect, |p| p.print((3, i + 1 - self.offset), symbol))
                    });
                    // Draw the active row.
                    p.with_color(white(), |p| {
                        p.print(
                            (6, i + 1 - self.offset),
                            format!("{:02}  {}", f.track, f.title).as_str(),
                        );
                        if column > 11 && (self.player.is_randomized || self.player.is_muted) {
                            // Draw the player options.
                            p.with_color(cyan(), |p| {
                                p.with_effect(Effect::Italic, |p| {
                                    p.print((column - 3, i + 1 - self.offset), self.player_opts())
                                })
                            })
                        }
                        p.print(
                            (column, i + 1 - self.offset),
                            mins_and_secs(f.duration).as_str(),
                        );
                    })
                } else if i + 2 - self.offset < h {
                    // Draw the inactive rows.
                    p.with_color(blue(), |p| {
                        p.print(
                            (6, i + 1 - self.offset),
                            format!("{:02}  {}", f.track, f.title).as_str(),
                        );
                        p.print(
                            (column, i + 1 - self.offset),
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

        if h > 1 {
            // Draw the header: 'Artist, Album, Year'.
            p.with_effect(Effect::Bold, |printer| {
                printer.with_color(green(), |printer| printer.print((2, 0), &f.artist.as_str()));
                printer.with_effect(Effect::Italic, |printer| {
                    printer.with_color(yellow(), |printer| {
                        printer.print((f.artist.len() + 4, 0), &self.album_and_year().as_str())
                    })
                })
            });

            if self.player.showing_volume {
                let column = if w > 14 { column - 5 } else { column };
                p.with_color(grey(), |p| p.print((column, 0), &self.volume(w).as_str()));
            };

            // The last row we can draw on.
            let last_row = h - 1;

            // Draw the elapsed and remaining playback times.
            p.with_color(white(), |printer| {
                let remaining = min(f.duration, f.duration - elapsed);
                printer.print((0, last_row), &mins_and_secs(elapsed));
                printer.print((column, last_row), mins_and_secs(remaining).as_str())
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
    }

    fn layout(&mut self, size: cursive::Vec2) {
        self.player.poll_sink();
        if self.player.is_randomized && self.player.is_queued {
            self.random_track();
        }
        self.size = size;
        self.offset = self.update_offset();
    }

    // Keybindings for the player view.
    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('g') => self.player.play_selection(),
            Event::CtrlChar('g') => self.player.play_last_track(),

            #[allow(unused_variables)]
            Event::Mouse {
                offset: XY { x, y },
                event: MouseEvent::Press(MouseButton::Left),
                ..
            } => return self.mouse_select(y, event),

            Event::Char('h') | Event::Char(' ') | Event::Key(Key::Left) => {
                let status = self.player.play_or_pause();

                return match self.cb {
                    Some(_) => EventResult::with_cb(move |siv| {
                        siv.with_user_data(|(opts, _, _): &mut UserData| {
                            opts.0 = status;
                        });
                    }),
                    None => EventResult::Consumed(None),
                };
            }

            Event::Char('l')
            | Event::Key(Key::Enter)
            | Event::Key(Key::Right)
            | Event::Mouse {
                event: MouseEvent::Press(MouseButton::Right),
                ..
            } => {
                let status = self.player.stop();

                return match self.cb {
                    Some(_) => EventResult::with_cb(move |siv| {
                        siv.with_user_data(|(opts, _, _): &mut UserData| {
                            opts.0 = status;
                        });
                    }),
                    None => EventResult::Consumed(None),
                };
            }

            Event::Char('j')
            | Event::Key(Key::Down)
            | Event::Mouse {
                event: MouseEvent::WheelDown,
                ..
            } => {
                if self.player.is_randomized {
                    self.random_track();
                } else {
                    self.player.next();
                }
            }

            Event::Char('k')
            | Event::Key(Key::Up)
            | Event::Mouse {
                event: MouseEvent::WheelUp,
                ..
            } => {
                if self.player.is_randomized {
                    self.previous_random();
                } else {
                    self.player.previous()
                }
            }

            Event::Char(']') => {
                let volume = self.player.increase_volume();

                return match self.cb {
                    Some(_) => EventResult::with_cb(move |siv| {
                        siv.with_user_data(|(opts, _, _): &mut UserData| {
                            opts.1 = volume;
                        });
                    }),
                    None => EventResult::Consumed(None),
                };
            }
            Event::Char('[') => {
                let volume = self.player.decrease_volume();

                return match self.cb {
                    Some(_) => EventResult::with_cb(move |siv| {
                        siv.with_user_data(|(opts, _, _): &mut UserData| {
                            opts.1 = volume;
                        });
                    }),
                    None => EventResult::Consumed(None),
                };
            }
            Event::Char('v') => self.player.show_volume(),
            Event::Char('m') => {
                let is_muted = self.player.toggle_mute();

                return match self.cb {
                    Some(_) => EventResult::with_cb(move |siv| {
                        siv.with_user_data(|(opts, _, _): &mut UserData| {
                            opts.2 = is_muted;
                        });
                    }),
                    None => EventResult::Consumed(None),
                };
            }

            Event::Char('*' | 'r') => {
                if self.cb.is_none() && self.player.playlist.len() < 2 {
                    return EventResult::Consumed(None);
                }

                self.player.toggle_randomization();

                if self.player.is_randomized {
                    let current_index = self.player.index;

                    return match self.cb {
                        Some(_) => EventResult::with_cb(move |siv| {
                            siv.with_user_data(|(_, _, queue): &mut UserData| {
                                if let Some((_, index)) = queue.get_mut(1) {
                                    *index = current_index;
                                }
                            });
                        }),
                        None => {
                            if self.player.playlist.len() > 1 {
                                EventResult::with_cb(move |siv| siv.set_user_data(current_index))
                            } else {
                                EventResult::Consumed(None)
                            }
                        }
                    };
                }
            }

            Event::Char('0') => self.player.number_keys.push(0),
            Event::Char('1') => self.player.number_keys.push(1),
            Event::Char('2') => self.player.number_keys.push(2),
            Event::Char('3') => self.player.number_keys.push(3),
            Event::Char('4') => self.player.number_keys.push(4),
            Event::Char('5') => self.player.number_keys.push(5),
            Event::Char('6') => self.player.number_keys.push(6),
            Event::Char('7') => self.player.number_keys.push(7),
            Event::Char('8') => self.player.number_keys.push(8),
            Event::Char('9') => self.player.number_keys.push(9),

            Event::Char('?') => {
                return EventResult::with_cb(|siv| {
                    KeysView::load(siv);
                });
            }

            Event::Char('q') => {
                return EventResult::with_cb(|siv| {
                    siv.quit();
                });
            }

            _ => return EventResult::Ignored,
        }

        self.selected = None;
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

// Remove all layers from the StackView except the top layer.
fn remove_layers_to_top(siv: &mut Cursive) {
    let mut count = siv.screen().len();

    while count > 1 {
        siv.screen_mut()
            .remove_layer(cursive::views::LayerPosition::FromBack(0));
        count -= 1;
    }
}
