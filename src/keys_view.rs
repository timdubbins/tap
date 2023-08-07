use cursive::{
    align::HAlign,
    event::{Event, EventTrigger, MouseEvent},
    utils::markup::StyledString,
    view::Resizable,
    views::{
        Button, Dialog, DummyView, Layer, LinearLayout, ListView, OnEventView, PaddedView,
        ScrollView, TextView,
    },
    Cursive, With,
};

use crate::theme;

pub struct KeysView {}

impl KeysView {
    pub fn new() -> PaddedView<ScrollView<LinearLayout>> {
        PaddedView::lrtb(
            2,
            2,
            2,
            2,
            ScrollView::new(
                LinearLayout::vertical()
                    .child(
                        Dialog::new().title("Global").content(
                            ListView::new()
                                .child("fuzzy search", TextView::new("`Tab`"))
                                .child("filtered search", TextView::new("`A...Z`"))
                                .child("sorted search", TextView::new("`Ctrl` + `s`"))
                                .child("previous selection", TextView::new("`-`"))
                                .child("random selection", TextView::new("`=`")),
                        ),
                    )
                    .child(DummyView.fixed_height(1))
                    .child(
                        Dialog::new().title("Player").content(
                            ListView::new()
                                .child("play", TextView::new("`h` or `Left` or `Space`"))
                                .child("next", TextView::new("`j` or `Down`"))
                                .child("previous", TextView::new("`k` or `Up`"))
                                .child("stop", TextView::new("`l` or `Right` or `Enter`"))
                                .child("go to first track", TextView::new("`gg`"))
                                .child("go to last track", TextView::new("`Ctrl` + `g`"))
                                .child("go to track number", TextView::new("`0...9` + `g`"))
                                .child("mute", TextView::new("`m`"))
                                .child("help", TextView::new("`?`"))
                                .child("quit", TextView::new("`q`")),
                        ),
                    )
                    .child(DummyView.fixed_height(1))
                    .child(
                        Dialog::new().title("Fuzzy").content(
                            ListView::new()
                                .child("clear search", TextView::new("`Ctrl` + `u`"))
                                .child("cancel search     ", TextView::new("`Esc`"))
                                .child("page up", TextView::new("`Ctrl` + `h` or `PgUp`"))
                                .child("page down", TextView::new("`Ctrl` + `l` or `PgDn`"))
                                .child("random page", TextView::new("`Ctrl` + `z`")),
                        ),
                    )
                    .child(DummyView.fixed_height(2))
                    .child(
                        TextView::new(StyledString::styled(" <Back> ", theme::button())).center(),
                    ),
            )
            .show_scrollbars(false),
        )
    }

    pub fn load(siv: &mut Cursive) {
        siv.add_layer(
            OnEventView::new(KeysView::new()).on_event(KeysView::trigger(), |siv| {
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
