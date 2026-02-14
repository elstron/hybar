mod client;
mod config;
mod models;
mod ui;
mod user;
mod utils;
use gtk::{Application, ApplicationWindow, prelude::*};
use gtk4_layer_shell::LayerShell;
use std::{cell::Cell, env, path::PathBuf, rc::Rc, sync::Arc};

use client::hyprland_event_listener;
use config::{bootstrap::bootstrap_config, hidden_layer_configuration, layer_shell_configure};
use panels::settings::HasSettingsEvent;
use ui::{
    fullscreen::handle_fullscreen_visibility,
    sections::{BarSections, create_sections},
    widgets::WidgetsBuilder,
};
use user::config::load_config;
use utils::css::load_css;

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
}

pub struct EventState {
    pending_title: parking_lot::Mutex<Option<String>>,
}

#[derive(Clone, Debug)]
pub struct UiEventState {
    sender: async_channel::Sender<UiEvent>,
    theme: String,
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
}

#[tokio::main]
async fn main() {
    let app = Application::builder().application_id("com.hybar").build();

    app.connect_activate(|app| {
        if let Err(e) = bootstrap_config() {
            eprintln!("Error inicializando configuración: {e}");
        }

        let user_config = load_config().unwrap_or_default();
        let bar_height = 20;
        let user_config = Rc::new(user_config);

        load_css(&user_config.theme);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("hybar")
            .default_height(bar_height)
            .build();
        LayerShell::init_layer_shell(&window);

        let hidden_window = ApplicationWindow::builder()
            .application(app)
            .title("hidden hybar")
            .default_height(2)
            .build();
        LayerShell::init_layer_shell(&hidden_window);

        hidden_layer_configuration(&hidden_window, Rc::clone(&user_config));
        layer_shell_configure(&window, Rc::clone(&user_config));

        let BarSections {
            left: section_left,
            right: section_right,
            center: section_center,
            container: section_container,
        } = create_sections();

        let section_left = Rc::new(section_left);
        let section_center = Rc::new(section_center);
        let section_right = Rc::new(section_right);

        let (sender, receiver) = async_channel::unbounded::<UiEvent>();
        let is_window_visible = Rc::new(Cell::new(!user_config.bar.autohide));
        let user_config = Rc::new(user_config);
        let event_state = Arc::new(EventState::new());

        let sender_event = UiEventState {
            sender: sender.clone(),
            theme: user_config.theme.clone(),
        };
        let mut widgets_builder = WidgetsBuilder::new(
            Rc::clone(&user_config),
            Arc::clone(&event_state),
            Rc::clone(&is_window_visible),
            sender_event,
        );
        let has_workspace_widget = widgets_builder.sync_widgets_layout(
            Rc::clone(&section_left),
            Rc::clone(&section_right),
            Rc::clone(&section_center),
        );

        if has_workspace_widget || widgets_builder.widget_exists("title") {
            let event_state_clone = Arc::clone(&event_state);

            tokio::spawn(async move {
                hyprland_event_listener(event_state_clone, sender).await;
            });
        }

        section_container.append(section_left.as_ref());
        section_container.append(section_center.as_ref());
        section_container.append(section_right.as_ref());
        window.set_child(Some(&section_container));

        let motion_controller_for_normal_window =
            layer_motion_controller(&window, &hidden_window, Rc::clone(&is_window_visible));
        let motion_controller_for_hidden_window =
            hidden_bar_motion_controller(&window, &hidden_window, Rc::clone(&is_window_visible));

        window.add_controller(motion_controller_for_normal_window.clone());
        hidden_window.add_controller(motion_controller_for_hidden_window.clone());

        if !user_config.bar.autohide {
            hidden_window.set_focusable(false);
            window.set_focusable(false);

            window.remove_controller(&motion_controller_for_normal_window);
            hidden_window.remove_controller(&motion_controller_for_hidden_window);

            is_window_visible.set(true);
        }

        let window_clone = window.clone();
        let hidden_window_clone = hidden_window.clone();
        let is_window_visible_clone = Rc::clone(&is_window_visible);
        let hidden_controller_clone = motion_controller_for_hidden_window.clone();
        let normal_controller_clone = motion_controller_for_normal_window.clone();

        let user_config = Rc::clone(&user_config);
        glib::MainContext::default().spawn_local(async move {
            while let Ok(msg) = receiver.recv().await {
                match msg {
                    UiEvent::FullscreenChanged(is_fullscreen) => handle_fullscreen_visibility(
                        &window_clone,
                        &hidden_window_clone,
                        Rc::clone(&is_window_visible_clone),
                        is_fullscreen,
                        user_config.bar.autohide,
                        &normal_controller_clone,
                        &hidden_controller_clone,
                    ),
                    UiEvent::TitleChanged(title) => {
                        widgets_builder.widgets.title.set_title(title.as_str())
                    }
                    UiEvent::ReloadSettings => {
                        let new_config = load_config().unwrap_or_default();
                        widgets_builder.update_config(Rc::new(new_config));
                        widgets_builder.sync_widgets_layout(
                            Rc::clone(&section_left),
                            Rc::clone(&section_right),
                            Rc::clone(&section_center),
                        );
                    }
                    UiEvent::ThemeChanged(theme) => load_css(&theme),
                    UiEvent::WorkspaceChanged => {
                        widgets_builder.widgets.workspaces.update(None);
                    }
                    UiEvent::WorkspaceUrgent(urgent) => {
                        widgets_builder.widgets.workspaces.update(Some(urgent))
                    }
                    UiEvent::WindowOpened((window_name, id)) => {
                        widgets_builder.update_active_clients();
                        let widget = find_child_by_name_or_id(
                            &widgets_builder.widgets.apps,
                            &window_name,
                            &id,
                        );

                        if let Some(widget) = widget {
                            widget.add_css_class("opened");
                        } else {
                            widgets_builder.create_widget_app(&window_name, &id, true);
                        }
                    }
                    UiEvent::WindowClosed(id) => {
                        widgets_builder.update_active_clients();
                        let widget =
                            find_child_by_name_or_id(&widgets_builder.widgets.apps, "", &id);
                        if let Some(widget) = widget {
                            let formatted_id = format!("_{}", id);
                            let widget_name =
                                &widget.widget_name().replace(formatted_id.as_str(), "");
                            println!("Window closed event for widget: {}", widget_name);
                            widget.set_widget_name(widget_name);

                            if !widget_name.contains("_") {
                                widget.remove_css_class("opened");

                                let is_favorite =
                                    user_config.widgets.get("apps").is_some_and(|app_config| {
                                        app_config.favorites.clone().is_some_and(|favorites| {
                                            favorites.contains(widget_name)
                                        })
                                    });

                                if !is_favorite {
                                    widgets_builder
                                        .widgets
                                        .apps
                                        .clone()
                                        .downcast::<gtk::Box>()
                                        .unwrap()
                                        .remove(&widget);
                                }
                            }
                        }
                    }
                }
            }
        });
        hidden_window.present();
        window.present();
    });

    app.run();
}

fn find_child_by_name_or_id(box_: &gtk::Widget, name: &str, id: &str) -> Option<gtk::Widget> {
    let mut child = box_.first_child();
    while let Some(widget) = child {
        let name = if name.is_empty() { id } else { name };
        let has_name = widget.widget_name().contains(name);
        let has_id = widget.widget_name().contains(id);

        if !has_name {
            child = widget.next_sibling();
            continue;
        }

        if !has_id {
            widget.set_widget_name(&format!("{}_{}", widget.widget_name(), id));
        }

        return Some(widget);
    }

    None
}

pub fn layer_motion_controller(
    bar: &ApplicationWindow,
    hidden_bar: &ApplicationWindow,
    is_visible: Rc<Cell<bool>>,
) -> gtk::EventControllerMotion {
    let motion_controller = gtk::EventControllerMotion::new();

    let bar_clone = bar.clone();
    let hidden_bar_clone_for_leave = hidden_bar.clone();
    motion_controller.connect_leave(move |_| {
        is_visible.set(false);
        bar_clone.hide();
        bar_clone.set_focusable(false);
        let hidden_bar_clone = hidden_bar_clone_for_leave.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(10), move || {
            hidden_bar_clone.set_focusable(true);
            glib::ControlFlow::Break
        });
    });
    motion_controller
}

pub fn hidden_bar_motion_controller(
    bar: &ApplicationWindow,
    hidden_bar: &ApplicationWindow,
    is_visible: Rc<Cell<bool>>,
) -> gtk::EventControllerMotion {
    let motion_controller = gtk::EventControllerMotion::new();
    let hidden_bar_clone = hidden_bar.clone();
    let bar_clone = bar.clone();

    motion_controller.connect_enter(move |_, _x, _y| {
        is_visible.set(true);
        hidden_bar_clone.set_focusable(false);
        bar_clone.present();
        bar_clone.set_focusable(true);
    });
    motion_controller
}

pub fn get_hypr_socket_path() -> Option<PathBuf> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").ok()?;
    let instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    Some(
        PathBuf::from(runtime_dir)
            .join("hypr")
            .join(instance)
            .join(".socket2.sock"),
    )
}

pub fn set_popover(button: &gtk::Button, child: gtk::Widget) {
    let popover = gtk::Popover::builder()
        .child(&child)
        .has_arrow(true)
        .position(gtk::PositionType::Bottom)
        .build();

    popover.set_parent(button);

    let motion = gtk::EventControllerMotion::new();

    let popover_show = popover.clone();
    motion.connect_enter(move |_, _, _| {
        popover_show.present();
    });

    let popover_hide = popover.clone();
    motion.connect_leave(move |_| {
        popover_hide.popdown();
    });

    button.add_controller(motion);
}
