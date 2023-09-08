use cursive::event::{Event, EventTrigger, MouseEvent};
use cursive::utils::markup::StyledString;
use cursive::view::Resizable;
use cursive::views::{
    FixedLayout, Layer, LinearLayout, OnEventView, OnLayoutView, ResizedView, TextView,
};
use cursive::{Cursive, Rect, Vec2, View};

use crate::theme;

pub struct ErrorView {}

impl ErrorView {
    pub fn new(content: String) -> ResizedView<OnLayoutView<FixedLayout>> {
        let mut content = StyledString::styled(content, theme::white());
        content.append_plain("  ");
        content.append(StyledString::styled(" <Ok> ", theme::button()));
        content.append_plain("  ");

        OnLayoutView::new(
            FixedLayout::new().child(
                Rect::from_point(Vec2::zero()),
                LinearLayout::horizontal()
                    .child(Layer::with_color(TextView::new(" [error]: "), theme::red()))
                    .child(TextView::new(content))
                    .full_width(),
            ),
            |layout, size: cursive::XY<usize>| {
                layout.set_child_position(0, Rect::from_size((0, size.y - 2), (size.x, 2)));
                layout.layout(size);
            },
        )
        .full_screen()
    }

    pub fn load(siv: &mut Cursive, err: anyhow::Error) {
        let content = err.to_string();
        siv.screen_mut()
            .add_transparent_layer(OnEventView::new(ErrorView::new(content)).on_event(
                ErrorView::trigger(),
                |siv| {
                    siv.pop_layer();
                },
            ));
    }

    pub fn trigger() -> EventTrigger {
        EventTrigger::from_fn(|event| {
            matches!(
                event,
                Event::Char(_)
                    | Event::Key(_)
                    | Event::Mouse {
                        event: MouseEvent::WheelUp | MouseEvent::WheelDown | MouseEvent::Press(_),
                        ..
                    }
            )
        })
    }
}
