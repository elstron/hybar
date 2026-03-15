mod client;
mod config;
mod enums;
mod hybar;
mod impls;
mod models;
mod ui;
mod user;
mod utils;

use gtk::{Application, prelude::*};
use hybar::{BarPreferences, Hybar};

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
    BarPositionChanged(String),
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

#[tokio::main]
async fn main() {
    let app = Application::builder().application_id("com.hybar").build();

    app.connect_activate(|app| {
        let hybar = Hybar::new(app);
        hybar.build()
    });

    app.run();
}
