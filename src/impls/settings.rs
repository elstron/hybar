use crate::{PreferencesEvent, UiEvent, UiEventState};
use panels::settings::traits::HasSettingsEvent;

impl HasSettingsEvent for UiEventState {
    fn pending_reload(&self) {
        self.sender
            .try_send(UiEvent::ReloadSettings)
            .unwrap_or_else(|e| eprintln!("Failed to send reload settings event: {}", e));
    }

    fn pending_theme_change(&self, theme: String) {
        self.sender
            .try_send(UiEvent::ThemeChanged(theme))
            .unwrap_or_else(|e| eprintln!("Failed to send theme change event: {}", e));
    }

    fn get_default_theme(&self) -> String {
        self.theme.clone()
    }

    fn enable_autohide(&self, _enable: bool) {
        self.sender
            .try_send(UiEvent::PreferencesChanged(
                PreferencesEvent::AutohideChanged(_enable),
            ))
            .unwrap_or_else(|e| eprintln!("Failed to send fullscreen change event: {}", e));
    }

    fn get_autohide(&self) -> bool {
        self.preferences.autohide
    }

    fn set_bar_position(&self, position: String) {
        self.sender
            .try_send(UiEvent::PreferencesChanged(
                PreferencesEvent::BarPositionChanged(position),
            ))
            .unwrap_or_else(|e| eprintln!("Failed to send bar position change event: {}", e));
    }

    fn get_bar_position(&self) -> String {
        self.preferences.bar_position.clone()
    }
}
