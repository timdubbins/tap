use cursive::{
    event::{Event, EventTrigger, MouseEvent},
    utils::markup::StyledString,
    view::Resizable,
    views::{FixedLayout, Layer, LinearLayout, OnEventView, OnLayoutView, ResizedView, TextView},
    Cursive, Rect, Vec2, View,
};

use crate::{config::ColorStyles, TapError};

// A Cursive view to display error messages.
pub struct ErrorView;

impl ErrorView {
    fn new(content: String) -> ResizedView<OnLayoutView<FixedLayout>> {
        let mut content = StyledString::styled(content, ColorStyles::hl());
        content.append_plain("  ");
        content.append(StyledString::styled(" <Ok> ", ColorStyles::err().invert()));
        content.append_plain("  ");

        OnLayoutView::new(
            FixedLayout::new().child(
                Rect::from_point(Vec2::zero()),
                LinearLayout::horizontal()
                    .child(Layer::with_color(
                        TextView::new(" [error]: "),
                        ColorStyles::err(),
                    ))
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

    pub fn load(siv: &mut Cursive, err: TapError) {
        let content = err.to_string();
        siv.screen_mut()
            .add_transparent_layer(OnEventView::new(Self::new(content)).on_event(
                Self::trigger(),
                |siv| {
                    siv.pop_layer();
                },
            ));
    }

    fn trigger() -> EventTrigger {
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
