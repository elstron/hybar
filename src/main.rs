mod client;
mod config;
mod hybar;
mod models;
mod ui;
mod user;
mod utils;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gtk::{Application, prelude::*};
use hybar::Hybar;

use panels::settings::HasSettingsEvent;

use hybar::BarPreferences;
pub const BACKGROUND_COLOR: &str = "#1a202c";
const HYPRLAND_SUBSCRIPTION: &str = r#"["subscribe", ["workspace", "fullscreen"]]"#;
const DEBOUNCE_MS: u64 = 50;

pub enum UiEvent {
    WorkspaceChanged,
    WorkspaceUrgent(String),
    FullscreenChanged(bool),
    TitleChanged(String),
    ReloadSettings,
    WindowOpened((String, String)),
    WindowClosed(String),
    ThemeChanged(String),
    PreferencesChanged(PreferencesEvent),
}

pub enum PreferencesEvent {
    Reload,
    ThemeChanged(String),
    AutohideChanged(bool),
}

pub struct EventState {
    pending_title: parking_lot::Mutex<Option<String>>,
}

#[derive(Clone)]
pub struct UiEventState {
    sender: async_channel::Sender<UiEvent>,
    theme: String,
    preferences: BarPreferences,
}

impl EventState {
    fn new() -> Self {
        Self {
            pending_title: parking_lot::Mutex::new(None),
        }
    }
}

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
}

#[tokio::main]
async fn main() {
    let app = Application::builder().application_id("com.hybar").build();
    app.connect_activate(|app| {
        let hybar = Hybar::new(app);
        hybar.build()
    });

    app.run();
}
