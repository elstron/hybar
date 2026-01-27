use gtk::prelude::*;
use gtk::{ApplicationWindow, Box as GtkBox, CheckButton, ComboBoxText, Orientation};
use gtk4_layer_shell::LayerShell;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub trait HasPendingReload: Send + Sync {
    fn pending_reload(&self) -> &AtomicBool;
}

pub fn render<S: HasPendingReload + 'static>(state: Arc<S>) -> ApplicationWindow {
    let settings_window = ApplicationWindow::builder()
        .title("Settings")
        .default_width(400)
        .build();

    LayerShell::init_layer_shell(&settings_window);
    settings_window.set_layer(gtk4_layer_shell::Layer::Overlay);
    settings_window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    settings_window.set_anchor(gtk4_layer_shell::Edge::Left, false);
    settings_window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    settings_window.set_anchor(gtk4_layer_shell::Edge::Bottom, false);

    let settings_panel = settings_panel(&settings_window, &state.clone());

    settings_window.set_child(Some(&settings_panel));
    settings_window.add_css_class("settings-window");

    settings_window
}

pub fn settings_panel<S: HasPendingReload + 'static>(
    window: &ApplicationWindow,
    state: &Arc<S>,
) -> GtkBox {
    let vbox = GtkBox::new(Orientation::Vertical, 10);

    let feature_toggle = CheckButton::with_label("Enable Feature X");
    vbox.append(&feature_toggle);

    let option_dropdown = ComboBoxText::new();
    option_dropdown.append_text("Option 1");
    option_dropdown.append_text("Option 2");
    option_dropdown.append_text("Option 3");
    option_dropdown.set_active(Some(0));

    let reload_button = gtk::Button::with_label("Reload Settings");
    reload_button.add_css_class("info");

    let save_button = gtk::Button::with_label("Save Settings");
    save_button.add_css_class("info");

    let cancel_button = gtk::Button::with_label("Cancel");
    cancel_button.add_css_class("cancel");

    let window_clone = window.clone();
    cancel_button.connect_clicked(move |_| {
        window_clone.hide();
    });

    let state_clone = state.clone();
    reload_button.connect_clicked(move |_| {
        println!("Reloading settings...");
        state_clone
            .pending_reload()
            .store(true, std::sync::atomic::Ordering::SeqCst);
    });

    vbox.append(&option_dropdown);
    vbox.append(&reload_button);
    vbox.append(&cancel_button);
    vbox.append(&save_button);
    vbox
}
