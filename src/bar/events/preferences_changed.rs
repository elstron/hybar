use crate::{PreferencesEvent, bar::Hybar, utils::css::load_css};

impl Hybar {
    pub fn preferences_changed(&self, preference: PreferencesEvent) {
        match preference {
            PreferencesEvent::Reload => {}
            PreferencesEvent::ThemeChanged(theme) => load_css(&theme),
            PreferencesEvent::AutohideChanged(autohide) => self.autohide_changed(autohide),
            PreferencesEvent::BarPositionChanged(position) => self.bar_position_changed(position),
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
}
