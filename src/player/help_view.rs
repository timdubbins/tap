use cursive::{
    event::{Event, EventTrigger, MouseEvent},
    view::Resizable,
    views::{
        Dialog, DummyView, LinearLayout, ListView, OnEventView, PaddedView, ScrollView, TextView,
    },
    Cursive,
};

// A Cursive view for displaying the keybindings.
pub struct HelpView;

impl HelpView {
    fn new() -> ScrollView<PaddedView<LinearLayout>> {
        ScrollView::new(PaddedView::lrtb(
            2,
            2,
            0,
            0,
            LinearLayout::vertical()
                .child(
                    Dialog::new().title("Global").content(
                        ListView::new()
                            .child("fuzzy search:", TextView::new("Tab"))
                            .child("depth search:", TextView::new("F1...F4"))
                            .child("filtered search:", TextView::new("A-Z"))
                            .child("artist search:", TextView::new("Ctrl + a"))
                            .child("album search:", TextView::new("Ctrl + s"))
                            .child("parent search:", TextView::new("Ctrl + p"))
                            .child("previous album:", TextView::new("-"))
                            .child("random album:", TextView::new("="))
                            .child("open file manager:", TextView::new("Ctrl + o")),
                    ),
                )
                .child(DummyView.fixed_height(1))
                .child(
                    Dialog::new().title("Player").content(
                        ListView::new()
                            .child("play or pause:", TextView::new("h or ← or Space"))
                            .child("next:", TextView::new("j or n or ↓"))
                            .child("previous:", TextView::new("k or p or ↑"))
                            .child("stop:", TextView::new("l or → or Enter"))
                            .child("step << / >>:", TextView::new(", / ."))
                            .child("seek to sec", TextView::new("0-9 + \""))
                            .child("seek to min", TextView::new("0-9 + \'"))
                            .child("random:", TextView::new("* or r"))
                            .child("shuffle:", TextView::new("~ or s"))
                            .child("volume down / up:", TextView::new("[ / ]"))
                            .child("show volume:", TextView::new("v"))
                            .child("mute:", TextView::new("m"))
                            .child("go to first track:", TextView::new("gg"))
                            .child("go to last track:", TextView::new("Ctrl + g"))
                            .child("go to track number:", TextView::new("0-9, g"))
                            .child("help:", TextView::new("?"))
                            .child("quit:", TextView::new("q")),
                    ),
                )
                .child(DummyView.fixed_height(1))
                .child(
                    Dialog::new().title("Fuzzy").content(
                        ListView::new()
                            .child("clear search:", TextView::new("Ctrl + u"))
                            .child("cancel search:", TextView::new("Esc"))
                            .child("page up:", TextView::new("Ctrl + h or PgUp"))
                            .child("page down:", TextView::new("Ctrl + l or PgDn"))
                            .child("random page:", TextView::new("Ctrl + z")),
                    ),
                ),
        ))
        .show_scrollbars(true)
    }

    pub fn load(siv: &mut Cursive) {
        siv.add_layer(
            OnEventView::new(Self::new()).on_event(Self::trigger(), |siv| {
                siv.pop_layer();
            }),
        )
    }

    fn trigger() -> EventTrigger {
        EventTrigger::from_fn(|event| {
            matches!(
                event,
                Event::Char(_)
                    | Event::Key(_)
                    | Event::Mouse {
                        event: MouseEvent::Press(_),
                        ..
                    }
            )
        })
    }
}
