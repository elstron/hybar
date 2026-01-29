use gtk::prelude::*;
use gtk::{glib::DateTime, ApplicationWindow, Calendar};
use gtk4_layer_shell::LayerShell;

pub fn render() -> ApplicationWindow {
    let calendar_window = ApplicationWindow::builder()
        .title("Calendar")
        .default_width(300)
        .default_height(400)
        .build();

    LayerShell::init_layer_shell(&calendar_window);
    calendar_window.set_layer(gtk4_layer_shell::Layer::Overlay);
    calendar_window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    calendar_window.set_anchor(gtk4_layer_shell::Edge::Left, false);
    calendar_window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    calendar_window.set_anchor(gtk4_layer_shell::Edge::Bottom, false);

    calendar_window.add_css_class("calendar-window");

    let calendar = Calendar::new();
    let now = DateTime::now_local().unwrap();
    calendar.select_day(&now);
    calendar_window.set_child(Some(&calendar));
    calendar_window
}
