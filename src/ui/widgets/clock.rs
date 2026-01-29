use chrono::Local;
use glib::ControlFlow;
use gtk::{Box as GtkBox, Button, Orientation, prelude::*};
use std::{cell::Cell, rc::Rc};

pub fn render(is_visible: &Rc<Cell<bool>>) -> gtk::Widget {
    let clock_label = gtk::Label::new(Some(Local::now().format("%I:%M %P").to_string().as_str()));

    let clock_container = Button::builder()
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();
    clock_container.add_css_class("clock-container");
    clock_container.set_child(Some(&clock_label));

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
    let calendar_window = panels::calendar::render();
    clock_container.connect_clicked(move |_| {
        if calendar_window.is_visible() {
            calendar_window.hide();
        } else {
            calendar_window.show();
        }
    });

    clock_container.into()
}
