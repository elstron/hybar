use directories::ProjectDirs;
use gtk::prelude::*;
use gtk::{ApplicationWindow, Box as GtkBox, CheckButton, ComboBoxText, Orientation};
use gtk4_layer_shell::LayerShell;
use std::fs;
use std::sync::Arc;

#[derive(Clone)]
pub struct SettingsPanel {
    window: ApplicationWindow,
    state: Arc<dyn HasSettingsEvent>,
    settings: Settings,
}

#[derive(Clone)]
pub struct Settings {
    theme: String,
    height: u32,
}

pub trait HasSettingsEvent: Send + Sync {
    fn pending_reload(&self);
    fn pending_theme_change(&self, theme: String);
    fn get_default_theme(&self) -> String;
}

impl SettingsPanel {
    pub fn new<S: HasSettingsEvent + 'static>(state: S) -> Self {
        let window = ApplicationWindow::builder()
            .title("Settings")
            .default_width(400)
            .build();

        let state = Arc::new(state);
        Self {
            window,
            state: state.clone(),
            settings: Settings {
                theme: state.get_default_theme(),
                height: 30,
            },
        }
    }

    pub fn render(&self) -> ApplicationWindow {
        let settings_window = self.window.clone();

        LayerShell::init_layer_shell(&settings_window);

        settings_window.auto_exclusive_zone_enable();
        settings_window.set_layer(gtk4_layer_shell::Layer::Overlay);
        settings_window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        settings_window.set_anchor(gtk4_layer_shell::Edge::Left, false);
        settings_window.set_anchor(gtk4_layer_shell::Edge::Top, true);
        settings_window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
        settings_window.set_namespace(Some("hybar:settings"));
        let settings_panel = self.settings_panel();

        settings_window.set_child(Some(&settings_panel));
        settings_window.add_css_class("settings-window");

        settings_window
    }

    fn settings_panel(&self) -> GtkBox {
        let vbox = GtkBox::new(Orientation::Vertical, 20);

        let feature_toggle = CheckButton::with_label("Enable Feature X");
        feature_toggle.set_active(true);
        feature_toggle.add_css_class("feature-toggle");

        feature_toggle.connect_toggled(move |btn| match btn.is_active() {
            true => println!("Feature X enabled"),
            false => println!("Feature X disabled"),
        });

        vbox.append(&feature_toggle);

        let theme_selector = self.select_theme();

        let reload_button = gtk::Button::with_label("Reload Settings");
        reload_button.add_css_class("info");

        let save_button = gtk::Button::with_label("Save Settings");
        save_button.add_css_class("info");

        let cancel_button = gtk::Button::with_label("Cancel");
        cancel_button.add_css_class("cancel");

        let window_clone = self.window.clone();
        cancel_button.connect_clicked(move |_| {
            window_clone.hide();
        });

        let state_clone = Arc::clone(&self.state);
        reload_button.connect_clicked(move |_| state_clone.pending_reload());

        vbox.append(&theme_selector);
        vbox.append(&reload_button);
        vbox.append(&cancel_button);
        vbox.append(&save_button);
        vbox
    }

    fn themes_list(&self) -> Vec<String> {
        let proj_dirs = ProjectDirs::from("com", "stron", "hybar")
            .expect("The config directory could not be determined.");
        let themes_dir = proj_dirs.config_dir().join("themes");

        let mut themes = Vec::new();
        if !themes_dir.exists() && !themes_dir.is_dir() {
            eprintln!("Themes directory does not exist: {}", themes_dir.display());
            return themes;
        }

        for entry in fs::read_dir(themes_dir).expect("Failed to read themes directory") {
            let entry = entry.expect("Failed to read theme entry");

            if !entry.path().is_file() {
                continue;
            }

            if let Some(name) = entry.file_name().to_str() {
                themes.push(name.to_string().replace(".css", ""));
            }
        }
        themes
    }

    fn select_theme(&self) -> GtkBox {
        let hbox = GtkBox::new(Orientation::Horizontal, 5);
        let label = gtk::Label::new(Some("Select Theme:"));
        let option_dropdown = ComboBoxText::new();
        let themes = self.themes_list();
        for (i, theme) in themes.iter().enumerate() {
            option_dropdown.append_text(theme);
            if theme.clone() == self.state.get_default_theme() {
                option_dropdown.set_active(Some(i as u32));
            }
        }
        option_dropdown.add_css_class("option-dropdown");
        option_dropdown.connect_changed({
            let state_clone = Arc::clone(&self.state);
            move |combo| {
                if let Some(theme) = combo.active_text() {
                    state_clone.pending_theme_change(theme.to_string());
                }
            }
        });
        hbox.append(&label);
        hbox.append(&option_dropdown);
        hbox.add_css_class("section");
        hbox
    }

    fn save_settings(&self) {
        println!("Saving settings...");
    }
}
