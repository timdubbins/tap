use cursive::Cursive;

// Remove all layers from the StackView except the top layer.
pub fn remove_layers_to_top(siv: &mut Cursive) {
    let mut count = siv.screen().len();

    while count > 1 {
        siv.screen_mut()
            .remove_layer(cursive::views::LayerPosition::FromBack(0));
        count -= 1;
    }
}

// Pop all layers from the StackView except the bottom layer.
pub fn pop_layers_to_bottom(siv: &mut Cursive) {
    let mut count = siv.screen().len();

    while count > 1 {
        siv.pop_layer();
        count -= 1;
    }
}
