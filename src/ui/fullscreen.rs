use gtk::{ApplicationWindow, prelude::*};
use std::{cell::Cell, rc::Rc};

pub fn handle_fullscreen_visibility(
    window: &ApplicationWindow,
    hidden_window: &ApplicationWindow,
    is_window_visible: Rc<Cell<bool>>,
    is_fullscreen: bool,
) {
    if is_fullscreen {
        window.hide();
        hidden_window.set_focusable(true);
        is_window_visible.set(false);
    }

    if !is_fullscreen {
        hidden_window.set_focusable(false);
        window.present();
        is_window_visible.set(true);
    }
}
