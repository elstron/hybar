use chrono::Local;
use glib::ControlFlow;
use gtk::{Orientation, prelude::*};
use std::{cell::Cell, rc::Rc};

pub fn render(is_visible: &Rc<Cell<bool>>) -> gtk::Widget {
    let clock_container = gtk::Box::new(Orientation::Horizontal, 5);
    clock_container.add_css_class("clock-container");
    let clock_label = gtk::Label::new(Some(Local::now().format("%I:%M %P").to_string().as_str()));
    clock_container.append(&clock_label);

    let clock_label = Rc::new(clock_label);
    let is_visible = Rc::clone(is_visible);
    glib::timeout_add_local(std::time::Duration::from_secs(1), {
        let clock_label = Rc::clone(&clock_label);
        move || {
            if is_visible.get() {
                let now = chrono::Local::now();
                clock_label.set_label(&now.format("%I:%M %P").to_string());
            }
            ControlFlow::Continue
        }
    });

    clock_container.set_tooltip_markup(Some(&Local::now().format("%A, %B %d, %Y").to_string()));
    clock_container.into()
}
