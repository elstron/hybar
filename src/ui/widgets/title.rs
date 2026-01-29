use std::{rc::Rc, sync::Arc, time::Duration};

use glib::ControlFlow;
use gtk::{Orientation, pango, prelude::*};

pub trait HasPendingTitle: Send + Sync {
    fn pending_title(&self) -> &parking_lot::Mutex<Option<String>>;
}

pub fn render<S: HasPendingTitle + 'static>(event_state: Arc<S>) -> gtk::Widget {
    let title_container = gtk::Box::new(Orientation::Horizontal, 5);
    title_container.add_css_class("title-container");

    let title_label = gtk::Label::new(Some(""));
    title_label.set_ellipsize(pango::EllipsizeMode::End);
    title_label.set_max_width_chars(100);
    let title_label = Rc::new(title_label);
    title_container.append(title_label.as_ref());

    glib::timeout_add_local(Duration::from_millis(100), move || {
        if let Some(new_title) = event_state.pending_title().lock().take() {
            title_label.set_text(&new_title);
        }
        ControlFlow::Continue
    });

    title_container.into()
}
