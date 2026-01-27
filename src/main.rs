use glib::ControlFlow::{self};
use gtk::gdk::Cursor;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Orientation, gio, pango};
use gtk4_layer_shell::LayerShell;
mod config;
mod utils;
mod widgets;
use config::{hidden_layer_configuration, layer_shell_configure};
use settings::HasPendingReload;
use utils::css::load_css;
mod client;
mod user;
use chrono::Local;
use client::hyprland_event_listener;
use std::{
    cell::Cell,
    collections::HashSet,
    env,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use user::config::load_config;
use user::models::UserConfig;
use widgets::workspaces;
use widgets::workspaces::HasPendingWorkspace;

pub const BACKGROUND_COLOR: &str = "#1a202c";
const HYPRLAND_SUBSCRIPTION: &str = r#"["subscribe", ["workspace", "fullscreen"]]"#;
const DEBOUNCE_MS: u64 = 50;

#[derive(Debug)]
pub struct BarSections {
    left: GtkBox,
    right: GtkBox,
    center: GtkBox,
    container: GtkBox,
}

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

        let has_workspace_widget = build_widgets(
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

                build_widgets(
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

pub fn build_widgets(
    widgets_cache: &Rc<std::cell::RefCell<std::collections::HashMap<String, gtk::Widget>>>,
    user_config: &UserConfig,
    section_left: Rc<GtkBox>,
    section_right: Rc<GtkBox>,
    section_center: Rc<GtkBox>,
    event_state: &Arc<EventState>,
    is_window_visible: &Rc<Cell<bool>>,
) -> bool {
    let has_workspace_widget_left = create_widgets(
        Rc::clone(widgets_cache),
        &user_config.sections.left,
        &section_left,
        user_config,
        Arc::clone(event_state),
        Rc::clone(is_window_visible),
    );
    let has_workspace_widget_center = create_widgets(
        Rc::clone(widgets_cache),
        &user_config.sections.center,
        &section_center,
        user_config,
        Arc::clone(event_state),
        Rc::clone(is_window_visible),
    );
    let has_workspace_widget_right = create_widgets(
        Rc::clone(widgets_cache),
        &user_config.sections.right,
        &section_right,
        user_config,
        Arc::clone(event_state),
        Rc::clone(is_window_visible),
    );
    has_workspace_widget_left || has_workspace_widget_center || has_workspace_widget_right
}

fn create_widgets(
    widgets_cache: Rc<std::cell::RefCell<std::collections::HashMap<String, gtk::Widget>>>,
    widgets: &[String],
    container: &Rc<GtkBox>,
    config: &UserConfig,
    event_state: Arc<EventState>,
    is_visible: Rc<Cell<bool>>,
) -> bool {
    let mut has_workspace = false;
    let mut last_widget: Option<gtk::Widget> = None;
    let mut active_widgets: HashSet<gtk::Widget> = HashSet::new();
    for item in widgets.iter() {
        if item == "workspaces" {
            has_workspace = true;
        }

        let widget = widgets_cache
            .borrow_mut()
            .entry(item.to_string())
            .or_insert_with(|| {
                get_widget(
                    item,
                    config,
                    Arc::clone(&event_state),
                    Rc::clone(&is_visible),
                )
            })
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

fn widget_exists(config: &UserConfig, widget_name: &str) -> bool {
    config.sections.left.contains(&widget_name.to_string())
        || config.sections.center.contains(&widget_name.to_string())
        || config.sections.right.contains(&widget_name.to_string())
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

pub fn get_widget(
    name: &str,
    config: &UserConfig,
    event_state: Arc<EventState>,
    is_visible: Rc<Cell<bool>>,
) -> gtk::Widget {
    match name {
        "separator" => {
            let icon = config.widgets.get("separator").and_then(|w| w.icon.clone());
            let separator = gtk::Label::new(Some(icon.as_deref().unwrap_or("\u{f078}")));
            separator.add_css_class("separator");
            separator.into()
        }
        "workspaces" => workspaces::workspaces_build(Arc::clone(&event_state)),
        "clock" => {
            let clock_container = gtk::Box::new(Orientation::Horizontal, 5);
            clock_container.add_css_class("clock-container");
            let clock_label =
                gtk::Label::new(Some(Local::now().format("%I:%M %P").to_string().as_str()));
            clock_container.append(&clock_label);

            let clock_label = Rc::new(clock_label);
            let is_visible = Rc::clone(&is_visible);
            glib::timeout_add_local(std::time::Duration::from_secs(15), {
                let clock_label = Rc::clone(&clock_label);
                move || {
                    if is_visible.get() {
                        let now = chrono::Local::now();
                        clock_label.set_label(&now.format("%I:%M %P").to_string());
                    }
                    ControlFlow::Continue
                }
            });

            clock_container
                .set_tooltip_markup(Some(&Local::now().format("%A, %B %d, %Y").to_string()));

            clock_container.into()
        }
        "title" => {
            let title_container = gtk::Box::new(Orientation::Horizontal, 5);
            title_container.add_css_class("title-container");

            let title_label = gtk::Label::new(Some(""));
            title_label.set_ellipsize(pango::EllipsizeMode::End);
            title_label.set_max_width_chars(100);
            let title_label = Rc::new(title_label);
            title_container.append(title_label.as_ref());

            glib::timeout_add_local(Duration::from_millis(100), move || {
                if let Some(new_title) = event_state.pending_title.lock().take() {
                    title_label.set_text(&new_title);
                }
                ControlFlow::Continue
            });

            title_container.into()
        }
        "settings" => {
            let settings_button = gtk::Button::with_label("");
            settings_button.add_css_class("settings-button");

            let window = settings::render(Arc::clone(&event_state));
            settings_button.connect_clicked(move |_| {
                println!("Opening settings window");
                window.present();
            });
            settings_button.into()
        }
        _ => {
            let button = config.custom_apps.get(name);
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
                        if let Some(app) = gio::DesktopAppInfo::new(&cmd) {
                            let _ = app.launch(&[], None::<&gio::AppLaunchContext>);
                        } else {
                            let _ = std::process::Command::new("sh").arg("-c").arg(&cmd).spawn();
                        }
                    });
                }
                if button.tooltip.unwrap_or(true) {
                    btn.set_tooltip_text(Some(button.name.as_deref().unwrap_or("")));
                }

                btn.into()
            } else {
                gtk::Label::new(Some("Unknown")).into()
            }
        }
    }
}
fn create_sections() -> BarSections {
    let section_left = gtk::Box::new(Orientation::Horizontal, 0);
    section_left.set_halign(gtk::Align::Start);
    let section_right = gtk::Box::new(Orientation::Horizontal, 0);
    section_right.set_halign(gtk::Align::End);
    let section_center = gtk::Box::new(Orientation::Horizontal, 0);
    section_center.set_halign(gtk::Align::Center);
    section_center.add_css_class("section-center");

    let section_container = gtk::Box::new(Orientation::Horizontal, 10);
    section_container.set_homogeneous(true);
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
