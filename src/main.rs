use glib::ControlFlow::{self, Continue};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Orientation};
use gtk4_layer_shell::LayerShell;
mod config;
mod utils;
mod widgets;
use config::{hidden_layer_configuration, layer_shell_configure};
use serde::Deserialize;
use utils::css::load_css;
mod user;
use chrono::Local;
use std::cell::Cell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use user::config::load_config;

use crate::user::models::UserConfig;
use crate::widgets::workspaces::update_workspaces;

pub const BACKGROUND_COLOR: &str = "#1a202c";
// Hyprland socket subscription message
const HYPRLAND_SUBSCRIPTION: &str = r#"["subscribe", ["workspace", "fullscreen"]]"#;

#[derive(Deserialize)]
struct WidgetConfig {
    name: String,
}

#[derive(Deserialize)]
pub struct ConfigFile {
    widgets: Vec<WidgetConfig>,
}

#[derive(Debug)]
pub struct BarSections {
    left: GtkBox,
    right: GtkBox,
    center: GtkBox,
    container: GtkBox,
}
#[derive(Clone)]
pub enum Events {
    WorkspaceChange(String),
    FullscreenChange(),
    TitleChange(String),
}

#[tokio::main]
async fn main() {
    let app = Application::builder()
        .application_id("com.example.scale")
        .build();

    app.connect_activate(|app| {
        load_css();

        let user_config = load_config().unwrap_or_default();
        let bar_height = 20;

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Animación de Escala")
            .default_height(bar_height)
            .build();
        LayerShell::init_layer_shell(&window);

        let hidden_window = ApplicationWindow::builder()
            .application(app)
            .title("hidden window")
            .default_height(2)
            .build();
        LayerShell::init_layer_shell(&hidden_window);

        hidden_layer_configuration(&hidden_window);
        layer_shell_configure(&window);

        let BarSections {
            left: section_left,
            right: section_right,
            center: section_center,
            container: section_container,
        } = create_sections();

        // Use Rc to share sections without cloning
        let section_left = Rc::new(section_left);
        let section_center = Rc::new(section_center);
        let section_right = Rc::new(section_right);

        let (tx, _) = tokio::sync::broadcast::channel(128);
        
        // Centralized window state management
        // When autohide is false: window visible = true
        // When autohide is true: window starts hidden = false
        let is_window_visible = Rc::new(Cell::new(!user_config.bar.autohide));
        
        // Only create receivers for widgets that need them
        for widget in user_config.sections.right.iter() {
            let rx = if widget_needs_receiver(widget) {
                Some(tx.subscribe())
            } else {
                None
            };
            get_widget(widget, &section_right, &user_config, rx, Rc::clone(&is_window_visible));
        }

        for widget in user_config.sections.center.iter() {
            let rx = if widget_needs_receiver(widget) {
                Some(tx.subscribe())
            } else {
                None
            };
            get_widget(widget, &section_center, &user_config, rx, Rc::clone(&is_window_visible));
        }

        for widget in user_config.sections.left.iter() {
            let rx = if widget_needs_receiver(widget) {
                Some(tx.subscribe())
            } else {
                None
            };
            get_widget(widget, &section_left, &user_config, rx, Rc::clone(&is_window_visible));
        }
        let tx_clone = tx.clone();
        glib::MainContext::default().spawn_local(async move {
            if let Some(path) = get_hypr_socket_path() {
                match UnixStream::connect(path).await {
                    Ok(mut stream) => {
                        if let Err(e) = stream.write_all(HYPRLAND_SUBSCRIPTION.as_bytes()).await {
                            eprintln!("Failed to subscribe to Hyprland events: {}", e);
                            return;
                        }

                        let reader = BufReader::new(stream);
                        let mut lines = reader.lines();

                        while let Ok(Some(line)) = lines.next_line().await {
                            println!("Received: {}", line);
                            let is_workspace_change =
                                line.contains("\"change\":") || line.contains("workspace");
                            let is_title_change = line.contains("activewindow>>");
                            let is_fullscreen_change = line.contains("fullscreen") && line.contains("1");

                            if is_workspace_change {
                                tx_clone.send(Events::WorkspaceChange(line.clone())).ok();
                                continue;
                            }

                            if is_fullscreen_change {
                                tx_clone.send(Events::FullscreenChange()).ok();
                                continue;
                            }

                            if is_title_change {
                                let parts: Vec<&str> = line.split(">>").collect();
                                if parts.len() == 2 {
                                    let title = parts[1].trim().to_string();
                                    tx_clone.send(Events::TitleChange(title)).ok();
                                }
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to Hyprland socket: {}", e);
                    }
                }
            }
        });

        section_container.append(&section_left.as_ref());
        section_container.append(&section_center.as_ref());
        section_container.append(&section_right.as_ref());
        window.set_child(Some(&section_container));

        let motion_controller_for_normal_window = layer_motion_controller(&window, &hidden_window, Rc::clone(&is_window_visible));
        let motion_controller_for_hidden_window =
            hidden_bar_motion_controller(&window, &hidden_window, Rc::clone(&is_window_visible));

        println!("Autohide: {:?}", user_config.bar);
        
        // Improved fullscreen event handler
        let mut rx = tx.subscribe();
        glib::MainContext::default().spawn_local(async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    Events::FullscreenChange() => {
                        println!("Fullscreen event received");
                        // TODO: Implement auto-hide logic for fullscreen
                        // For now, just log the event and continue processing
                    }
                    _ => {
                        // Continue processing other events instead of terminating
                        continue;
                    }
                }
            }
        });
        if user_config.bar.autohide {
            window.add_controller(motion_controller_for_normal_window);
            hidden_window.add_controller(motion_controller_for_hidden_window);
            window.hide();
            hidden_window.present();
        } else {
            window.present();
        }
    });

    app.run();
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
        println!("Se salio de la zona de la barra");
    });
    motion_controller
}

pub fn hidden_bar_motion_controller(
    bar: &ApplicationWindow,
    hidden_bar: &ApplicationWindow,
    is_visible: Rc<Cell<bool>>,
) -> gtk::EventControllerMotion {
    bar.hide();

    let motion_controller = gtk::EventControllerMotion::new();

    let hidden_bar_clone = hidden_bar.clone();
    let bar_clone = bar.clone();

    motion_controller.connect_enter(move |_, _x, _y| {
        println!("Se entro en la zona de la barra oculta");
        is_visible.set(true);
        hidden_bar_clone.set_focusable(false);
        bar_clone.present();
        bar_clone.set_focusable(true);
    });
    motion_controller
}

pub fn get_widget(
    name: &str,
    container: &Rc<GtkBox>,
    config: &UserConfig,
    rkv: Option<tokio::sync::broadcast::Receiver<Events>>,
    is_visible: Rc<Cell<bool>>,
) {
    match name {
        "separator" => {
            let icon = config.widgets.get("separator").and_then(|w| w.icon.clone());
            let separator = gtk::Label::new(Some(icon.as_deref().unwrap_or("\u{f078}")));
            separator.add_css_class("separator");
            container.append(&separator);
        }
        "workspaces" => {
            if let Some(mut rkv) = rkv {
                let container = Rc::clone(container);
                update_workspaces(&container);

                glib::MainContext::default().spawn_local(async move {
                    while let Ok(event) = rkv.recv().await {
                        match event {
                            Events::WorkspaceChange(_) => update_workspaces(&container),
                            _ => continue,
                        }
                    }
                });
            } else {
                update_workspaces(container);
            }
        }
        "clock" => {
            println!("Agregando widget de reloj");
            let clock_label = gtk::Label::new(Some(
                Local::now().format("%I:%M:%S %P").to_string().as_str(),
            ));

            container.append(&clock_label);

            let clock_label = Rc::new(clock_label);
            // Optimize clock updates: only update when window is visible
            // Note: The timer continues to fire every second, but we skip the update
            // when the window is hidden. This trades consistent CPU wake-ups for simpler logic.
            // The overhead of checking a Cell<bool> is negligible (nanoseconds), while
            // stopping/restarting timers would require complex state management.
            glib::timeout_add_local(std::time::Duration::from_secs(1), {
                let clock_label = Rc::clone(&clock_label);
                move || {
                    // Only update the clock if the window is visible
                    if is_visible.get() {
                        let now = chrono::Local::now();
                        clock_label.set_label(&now.format("%I:%M:%S %P").to_string());
                    }
                    ControlFlow::Continue
                }
            });
        }
        "title" => {
            let title_label = gtk::Label::new(Some("Titulo de ventana"));

            if let Some(mut rkv) = rkv {
                let title_label_clone = title_label.clone();
                glib::MainContext::default().spawn_local(async move {
                    while let Ok(event) = rkv.recv().await {
                        match event {
                            Events::TitleChange(new_title) => {
                                title_label_clone.set_text(&new_title);
                            }
                            _ => continue,
                        }
                    }
                });
            }

            container.append(&title_label);
        }
        _ => {
            println!("Widget no reconocido: {}", name);
        }
    }
}

/// Determines if a widget needs an event receiver
fn widget_needs_receiver(widget_name: &str) -> bool {
    matches!(widget_name, "workspaces" | "title")
}

fn create_sections() -> BarSections {
    let section_left = gtk::Box::new(Orientation::Horizontal, 0);
    section_left.set_halign(gtk::Align::Start);
    let section_right = gtk::Box::new(Orientation::Horizontal, 0);
    section_right.set_halign(gtk::Align::End);
    let section_center = gtk::Box::new(Orientation::Horizontal, 0);
    section_center.set_halign(gtk::Align::Center);
    section_center.add_css_class("section-center");

    let section_container = gtk::Box::new(Orientation::Horizontal, 0);
    section_container.set_halign(gtk::Align::Fill);
    section_container.add_css_class("section-container");

    section_left.set_hexpand(true);
    section_center.set_hexpand(true);
    section_right.set_hexpand(true);

    BarSections {
        left: section_left,
        right: section_right,
        center: section_center,
        container: section_container,
    }
}

fn get_hypr_socket_path() -> Option<PathBuf> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").ok()?;
    let instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    Some(
        PathBuf::from(runtime_dir)
            .join("hypr")
            .join(instance)
            .join(".socket2.sock"),
    )
}
