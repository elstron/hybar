mod client;
mod config;
mod ui;
mod user;
mod utils;

use glib::ControlFlow;
use gtk::{Application, ApplicationWindow, prelude::*};
use gtk4_layer_shell::LayerShell;
use std::{
    cell::Cell,
    env,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use client::hyprland_event_listener;
use config::{hidden_layer_configuration, layer_shell_configure};
use settings::HasPendingReload;
use ui::{
    sections::{BarSections, create_sections},
    widgets::{
        sync_widgets_layout, title::HasPendingTitle, widget_exists, workspaces::HasPendingWorkspace,
    },
};
use user::config::load_config;
use utils::css::load_css;

pub const BACKGROUND_COLOR: &str = "#1a202c";
const HYPRLAND_SUBSCRIPTION: &str = r#"["subscribe", ["workspace", "fullscreen"]]"#;
const DEBOUNCE_MS: u64 = 50;

pub struct EventState {
    pending_workspace: AtomicBool,
    pending_fullscreen: AtomicBool,
    is_fullscreen: AtomicBool,
    pending_title: parking_lot::Mutex<Option<String>>,
    pending_workspace_urgent: parking_lot::Mutex<Option<String>>,
    pending_reload: AtomicBool,
}

impl EventState {
    fn new() -> Self {
        Self {
            pending_workspace: AtomicBool::new(false),
            pending_fullscreen: AtomicBool::new(false),
            is_fullscreen: AtomicBool::new(false),
            pending_title: parking_lot::Mutex::new(None),
            pending_workspace_urgent: parking_lot::Mutex::new(None),
            pending_reload: AtomicBool::new(false),
        }
    }
}
impl HasPendingTitle for EventState {
    fn pending_title(&self) -> &parking_lot::Mutex<Option<String>> {
        &self.pending_title
    }
}

impl HasPendingReload for EventState {
    fn pending_reload(&self) -> &AtomicBool {
        &self.pending_reload
    }
}

impl HasPendingWorkspace for EventState {
    fn pending_workspace(&self) -> &AtomicBool {
        &self.pending_workspace
    }

    fn pending_workspace_urgent(&self) -> &parking_lot::Mutex<Option<String>> {
        &self.pending_workspace_urgent
    }
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

        let section_left = Rc::new(section_left);
        let section_center = Rc::new(section_center);
        let section_right = Rc::new(section_right);

        let widgets_cache: Rc<std::cell::RefCell<std::collections::HashMap<String, gtk::Widget>>> =
            Rc::new(std::cell::RefCell::new(std::collections::HashMap::new()));
        let is_window_visible = Rc::new(Cell::new(!user_config.bar.autohide));

        let event_state = Arc::new(EventState::new());

        let has_workspace_widget = sync_widgets_layout(
            &widgets_cache,
            &user_config,
            Rc::clone(&section_left),
            Rc::clone(&section_right),
            Rc::clone(&section_center),
            &event_state,
            &is_window_visible,
        );
        if has_workspace_widget || widget_exists(&user_config, "title") {
            let event_state_clone = Arc::clone(&event_state);

            tokio::spawn(async move {
                hyprland_event_listener(event_state_clone).await;
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
            println!("Autohide disabled");
            hidden_window.set_focusable(false);
            window.set_focusable(false);

            window.remove_controller(&motion_controller_for_normal_window);
            hidden_window.remove_controller(&motion_controller_for_hidden_window);

            is_window_visible.set(true);
        }

        let window_clone = window.clone();
        let hidden_window_clone = hidden_window.clone();
        let is_window_visible_clone = Rc::clone(&is_window_visible);
        let event_state_clone = Arc::clone(&event_state);
        let hidden_controller_clone = motion_controller_for_hidden_window.clone();
        let normal_controller_clone = motion_controller_for_normal_window.clone();
        let widgets_cache = Rc::clone(&widgets_cache);

        glib::timeout_add_local(Duration::from_millis(100), move || {
            if event_state_clone
                .pending_fullscreen
                .swap(false, Ordering::Relaxed)
            {
                let is_fullscreen = event_state_clone.is_fullscreen.load(Ordering::Relaxed);
                handle_fullscreen_event(
                    &window_clone,
                    &hidden_window_clone,
                    Rc::clone(&is_window_visible_clone),
                    is_fullscreen,
                    user_config.bar.autohide,
                    &normal_controller_clone,
                    &hidden_controller_clone,
                );
            }

            if event_state_clone
                .pending_reload
                .swap(false, Ordering::Relaxed)
            {
                println!("Reloading configuration...");
                let new_config = load_config().unwrap_or_default();

                sync_widgets_layout(
                    &widgets_cache,
                    &new_config,
                    Rc::clone(&section_left),
                    Rc::clone(&section_right),
                    Rc::clone(&section_center),
                    &event_state_clone,
                    &is_window_visible_clone,
                );
            }
            ControlFlow::Continue
        });

        hidden_window.present();
        window.present();
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

pub fn handle_fullscreen_event(
    window: &ApplicationWindow,
    hidden_window: &ApplicationWindow,
    is_window_visible: Rc<Cell<bool>>,
    is_fullscreen: bool,
    autohide: bool,
    normal_controller: &gtk::EventControllerMotion,
    hidden_controller: &gtk::EventControllerMotion,
) {
    if is_fullscreen {
        window.hide();
        hidden_window.set_focusable(true);
        is_window_visible.set(false);

        if !autohide {
            window.add_controller(normal_controller.clone());
            hidden_window.add_controller(hidden_controller.clone());
        }
    }

    if !is_fullscreen {
        hidden_window.set_focusable(false);
        if !autohide {
            window.remove_controller(normal_controller);
            hidden_window.remove_controller(hidden_controller);
            window.present();
            is_window_visible.set(true);
        }
    }
}
