use gtk::prelude::*;
use gtk::{glib::DateTime, Calendar};

pub fn render() -> gtk::Widget {
    let calendar_window = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let calendar = Calendar::new();
    let now = DateTime::now_local().unwrap();

    calendar.select_day(&now);
    calendar_window.append(&calendar);
    calendar_window.into()
}
