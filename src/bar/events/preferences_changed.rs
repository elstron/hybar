use crate::{PreferencesEvent, bar::Hybar, utils::css::load_css};
use gtk::prelude::GtkWindowExt;

impl Hybar {
    pub fn preferences_changed(&self, preference: PreferencesEvent) {
        match preference {
            PreferencesEvent::Reload => {}
            PreferencesEvent::ThemeChanged(theme) => load_css(&theme),
            PreferencesEvent::AutohideChanged(autohide) => self.autohide_changed(autohide),
            PreferencesEvent::BarPositionChanged(position) => self.bar_position_changed(position),
            PreferencesEvent::BarHeightChanged(height) => self.bar_height_changed(height),
        }
    }

    fn bar_position_changed(&self, position: String) {
        self.window.set_bar_position(&position);
        self.preferences.borrow_mut().bar_position = position;
    }

    fn autohide_changed(&self, autohide: bool) {
        self.window.toggle_autohide(autohide);
        self.preferences.borrow_mut().autohide = autohide;
    }

    fn bar_height_changed(&self, height: i32) {
        self.window.main.set_default_height(height);
        self.preferences.borrow_mut().height = height;
    }
}
