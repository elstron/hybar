use gtk::{ApplicationWindow, prelude::*};
use std::{cell::Cell, rc::Rc};

pub fn handle_fullscreen_visibility(
    window: &ApplicationWindow,
    hidden_window: &ApplicationWindow,
    is_window_visible: Rc<Cell<bool>>,
    is_fullscreen: bool,
    autohide: bool,
    normal_controller: &gtk::EventControllerMotion,
    hidden_controller: &gtk::EventControllerMotion,
) {
    if is_fullscreen {
        window.hide();
        hidden_window.set_focusable(true);
        is_window_visible.set(false);

        if !autohide {
            window.add_controller(normal_controller.clone());
            hidden_window.add_controller(hidden_controller.clone());
        }
    }

    if !is_fullscreen {
        hidden_window.set_focusable(false);
        if !autohide {
            window.remove_controller(normal_controller);
            hidden_window.remove_controller(hidden_controller);
            window.present();
            is_window_visible.set(true);
        }
    }
}
