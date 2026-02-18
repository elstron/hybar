pub mod clock;
pub mod separator;
pub mod title;
pub mod workspaces;

use gtk::{Box as GtkBox, Image, gdk::Cursor, prelude::*};
use std::{cell::Cell, collections::HashSet, path::Path, rc::Rc, sync::Arc};

use crate::{
    EventState, UiEventState,
    models::clients::Client,
    set_popover,
    user::models::{SectionsConfig, UserConfig},
    utils::{
        app_launch::app_lauch,
        clients::{self, focus_client},
    },
};

pub struct Widgets {
    pub workspaces: workspaces::WorkspacesWidget,
    pub clock: gtk::Widget,
    pub title: title::TitleWidget,
    pub apps: gtk::Widget,
}

pub struct WidgetsBuilder {
    user_config: Rc<UserConfig>,
    event_state: Arc<EventState>,
    is_visible: Rc<Cell<bool>>,
    pub widgets: Widgets,
    widgets_cache: Rc<std::cell::RefCell<std::collections::HashMap<String, gtk::Widget>>>,
    active_clients: Rc<std::cell::RefCell<Vec<Client>>>,
    sender: UiEventState,
}

impl WidgetsBuilder {
    pub fn new(
        user_config: Rc<UserConfig>,
        event_state: Arc<EventState>,
        is_visible: Rc<Cell<bool>>,
        sender: UiEventState,
    ) -> Self {
        Self {
            user_config: user_config.clone(),
            event_state: Arc::clone(&event_state),
            is_visible: is_visible.clone(),
            widgets: Widgets {
                workspaces: workspaces::WorkspacesWidget::new(),
                clock: clock::render(&is_visible),
                title: title::TitleWidget::new(),
                apps: gtk::Box::new(gtk::Orientation::Horizontal, 0).into(),
            },
            widgets_cache: Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())),
            active_clients: Rc::new(std::cell::RefCell::new(
                clients::active_clients().unwrap_or_default(),
            )),
            sender,
        }
    }

    pub fn build_widget(&self, name: &str) -> gtk::Widget {
        let name = name.split('_').next().unwrap_or(name);
        match name {
            "separator" => separator::render(&self.user_config),
            "workspaces" => self.widgets.workspaces.widget().clone().into(),
            "clock" => self.widgets.clock.clone(),
            "title" => self.widgets.title.widget().clone(),
            "shutdown" => {
                let button = gtk::Button::from_icon_name("system-shutdown");
                let label = gtk::Label::new(Some("Shutdown"));

                set_popover(&button, label.into());
                button.add_css_class("shutdown-button");
                button.into()
            }
            "player" => {
                let (window, status) = panels::player::build_ui();
                let button = gtk::Button::with_label("💤 No activity");

                let button_clone = button.clone();
                status.connect_notify_local(Some("label"), move |status, _| {
                    button_clone.set_label(&status.text());
                });

                button.add_css_class("player-button");
                button.connect_clicked(move |_| match window.is_visible() {
                    true => window.hide(),
                    false => window.present(),
                });
                button.into()
            }
            "apps" => {
                let container = &self.widgets.apps;
                container.add_css_class("apps-container");

                for app in &self
                    .user_config
                    .widgets
                    .get("apps")
                    .and_then(|w| w.favorites.as_ref())
                    .cloned()
                    .unwrap_or_default()
                {
                    self.create_widget_app(app, "", false);
                }
                container.clone()
            }
            "settings" => {
                let settings_button = gtk::Button::with_label("");
                settings_button.add_css_class("settings-button");

                let settings = panels::settings::SettingsPanel::new(self.sender.clone());
                let window = settings.render();

                settings_button.connect_clicked(move |_| match window.is_visible() {
                    true => window.hide(),
                    false => window.present(),
                });

                settings_button.into()
            }
            _ => {
                let button = self.user_config.custom_apps.get(name);
                if let Some(button) = button {
                    let btn = match button.icon.as_deref() {
                        Some(icon_name) => gtk::Button::with_label(icon_name),
                        None => gtk::Button::from_icon_name(
                            button.name.as_deref().unwrap_or("application-x-executable"),
                        ),
                    };
                    btn.add_css_class("custom-app");

                    let cursor = Cursor::from_name("pointer", None);
                    btn.set_cursor(cursor.as_ref());

                    if let Some(cmd) = &button.cmd {
                        let cmd = cmd.clone();
                        btn.connect_clicked(move |_| {
                            app_lauch(&cmd);
                        });
                    }
                    if button.tooltip.unwrap_or(true) {
                        let text = button.name.as_deref().unwrap_or("");
                        btn.set_tooltip_text(Some(text));
                    }

                    let name = button.name.as_deref().unwrap_or("custom-app");
                    btn.set_widget_name(name);
                    btn.into()
                } else {
                    gtk::Label::new(Some("")).into()
                }
            }
        }
    }

    pub fn sync_widgets_layout(
        &self,
        section_left: Rc<GtkBox>,
        section_right: Rc<GtkBox>,
        section_center: Rc<GtkBox>,
    ) -> bool {
        let SectionsConfig {
            left,
            right,
            center,
        } = &self.user_config.sections;

        let left = self.sync_section_widgets(&section_left, left.as_slice());
        let center = self.sync_section_widgets(&section_center, center.as_slice());
        let right = self.sync_section_widgets(&section_right, right.as_slice());

        left || center || right
    }

    fn sync_section_widgets(&self, container: &Rc<GtkBox>, widgets: &[String]) -> bool {
        let mut has_workspace = false;
        let mut last_widget: Option<gtk::Widget> = None;
        let mut active_widgets: HashSet<gtk::Widget> = HashSet::new();

        for item in widgets.iter() {
            if item == "workspaces" {
                has_workspace = true;
            }

            let widget = self
                .widgets_cache
                .borrow_mut()
                .entry(item.to_string())
                .or_insert_with(|| self.build_widget(item))
                .clone();
            if let Some(parent) = widget.parent()
                && parent != **container
            {
                widget.unparent();
            }

            widget.insert_after(&**container, last_widget.as_ref());
            active_widgets.insert(widget.clone());
            last_widget = Some(widget)
        }
        let mut child = container.first_child();
        while let Some(current) = child {
            let next = current.next_sibling();
            if !active_widgets.contains(&current) {
                current.unparent();
            }

            child = next;
        }
        has_workspace
    }

    pub fn widget_exists(&self, widget_name: &str) -> bool {
        let SectionsConfig {
            left,
            right,
            center,
        } = &self.user_config.sections;

        left.contains(&widget_name.to_string())
            || center.contains(&widget_name.to_string())
            || right.contains(&widget_name.to_string())
    }

    pub fn update_config(&mut self, user_config: Rc<UserConfig>) {
        self.user_config = user_config;
    }

    pub fn create_widget_app(&self, app_name: &str, id: &str, is_opened: bool) {
        let button = gtk::Button::from_icon_name(app_name);
        button.set_cursor(Cursor::from_name("pointer", None).as_ref());
        button.set_tooltip_text(Some(app_name));
        button.set_widget_name(app_name);
        button.add_css_class("app-button");

        if is_opened {
            button.add_css_class("opened");
        }

        let clients = self.active_clients.borrow();
        let app_clients_active = clients
            .iter()
            .filter(|c| c.class.to_lowercase().contains(app_name));

        let has_active_client = app_clients_active.clone().count() >= 1;

        if !id.is_empty() {
            button.set_widget_name(format!("{}_{}", app_name, id).as_str());
        } else if has_active_client {
            button.add_css_class("opened");
            let mut widget_name = app_name.to_string();
            for client in app_clients_active {
                widget_name.push_str(format!("_{}", &client.address.replace("0x", "")).as_str());
            }
            button.set_widget_name(widget_name.as_str());
        }

        let icon_theme = gtk::IconTheme::for_display(&gtk::gdk::Display::default().unwrap());
        let found_icon = icon_theme.has_icon(app_name);

        let mut exec = None;
        if !found_icon {
            use crate::utils::search::search_desktop_file;
            let icon_name = if let Some(desktop_file) = search_desktop_file(app_name) {
                exec = Some(desktop_file.exec.clone());
                desktop_file
                    .icon
                    .unwrap_or_else(|| "application-x-executable".to_string())
            } else {
                "application-x-executable".to_string()
            };
            button.set_child(Some(&self.load_icon(&icon_name, 16)));
        }

        let app_clone = app_name.to_string();
        let app_clients = Rc::clone(&self.active_clients);

        button.connect_clicked(move |_| {
            let exec_cmd = exec.clone().unwrap_or(app_clone.clone());
            let clients = app_clients.borrow();
            let mut iter = clients
                .iter()
                .filter(|c| c.class.to_lowercase().contains(&app_clone.to_lowercase()));

            match (iter.next(), iter.next()) {
                // temporary solution for multiple clients, should be improved in the future
                (Some(client), Some(_)) => focus_client(client),
                (Some(client), None) => focus_client(client),
                (None, _) => app_lauch(&exec_cmd),
            }
        });

        self.widgets
            .apps
            .clone()
            .downcast::<gtk::Box>()
            .unwrap()
            .append(&button);
    }

    pub fn update_active_clients(&self) {
        *self.active_clients.borrow_mut() = clients::active_clients().unwrap_or_default();
    }

    pub fn load_icon(&self, icon_name: &str, size: i32) -> Image {
        println!("Loading icon: {}", icon_name);
        let image = if icon_name.contains('/')
            || icon_name.ends_with(".png")
            || icon_name.ends_with(".svg")
            || icon_name.ends_with(".xpm")
        {
            if Path::new(icon_name).exists() {
                Image::from_file(icon_name)
            } else {
                Image::new()
            }
        } else {
            Image::from_icon_name(icon_name)
        };

        image.set_pixel_size(size);
        image
    }
}
