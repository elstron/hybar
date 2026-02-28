use gtk::{Application, ApplicationWindow, prelude::*};
use gtk4_layer_shell::LayerShell;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

use crate::{
    PreferencesEvent, UiEvent, UiEventState,
    client::HyprlandClient,
    config::{bootstrap::bootstrap_config, hidden_layer_configuration, layer_shell_configure},
    ui::{
        fullscreen::handle_fullscreen_visibility,
        sections::{BarSections, create_sections},
        widgets::WidgetsBuilder,
    },
    user::config::load_config,
    utils::css::load_css,
};

pub struct Hybar {
    window: BarWindows,
    preferences: Rc<RefCell<BarPreferences>>,
    widgets: Rc<Cell<WidgetsBuilder>>,
    channel: (
        async_channel::Sender<UiEvent>,
        async_channel::Receiver<UiEvent>,
    ),
}

#[derive(Clone)]
pub struct BarPreferences {
    pub autohide: bool,
    pub theme: String,
}

pub struct BarWindows {
    pub is_visible: Rc<Cell<bool>>,
    pub is_fullscreen: Rc<Cell<bool>>,
    pub main: ApplicationWindow,
    pub hidden: ApplicationWindow,
}

impl BarWindows {
    pub fn new(app: &Application) -> Self {
        let bar = Self {
            main: ApplicationWindow::new(app),
            hidden: ApplicationWindow::new(app),
            is_visible: Rc::new(Cell::new(true)),
            is_fullscreen: Rc::new(Cell::new(false)),
        };

        bar.main_window_settings();
        bar.hidden_window_settings();
        bar
    }

    fn main_window_settings(&self) {
        self.main.set_title(Some("hybar"));
        self.main.set_default_height(40);
        LayerShell::init_layer_shell(&self.main);

        layer_shell_configure(&self.main, "top");
    }

    fn hidden_window_settings(&self) {
        self.hidden.set_title(Some("hidden hybar"));
        self.hidden.set_default_height(2);
        LayerShell::init_layer_shell(&self.hidden);

        hidden_layer_configuration(&self.hidden, "top");
    }
}

impl Default for BarPreferences {
    fn default() -> Self {
        let config = load_config().unwrap_or_default();

        Self {
            autohide: config.bar.autohide,
            theme: "default".to_string(),
        }
    }
}

impl Hybar {
    pub fn new(app: &Application) -> Arc<Self> {
        let (sender, receiver) = async_channel::unbounded::<UiEvent>();
        let preferences = Rc::new(RefCell::new(BarPreferences::default()));
        let bar_window = BarWindows::new(app);
        let hidden_window = bar_window.main.clone();
        Self {
            window: bar_window,
            preferences: Rc::clone(&preferences),
            widgets: Rc::new(Cell::new(WidgetsBuilder::new(
                hidden_window,
                Rc::new(load_config().unwrap_or_default()),
                Arc::new(crate::EventState::new()),
                Rc::new(Cell::new(true)),
                UiEventState {
                    sender: sender.clone(),
                    theme: "default".to_string(),
                    preferences: preferences.borrow().clone(),
                },
            ))),
            channel: (sender, receiver),
        }
        .into()
    }

    pub fn build(self: Arc<Self>) {
        if let Err(e) = bootstrap_config() {
            eprintln!("Error inicializando configuración: {e}");
        }

        let user_config = load_config().unwrap_or_default();
        let bar_height = 40;
        let user_config = Rc::new(user_config);

        load_css(&user_config.theme);

        let window = self.window.main.clone();
        let hidden_window = self.window.hidden.clone();

        let BarSections {
            left: section_left,
            right: section_right,
            center: section_center,
            container: section_container,
        } = create_sections();

        let section_left = Rc::new(section_left);
        let section_center = Rc::new(section_center);
        let section_right = Rc::new(section_right);

        let is_window_visible = Rc::new(Cell::new(!user_config.bar.autohide));
        let user_config = Rc::new(user_config);
        let event_state = Arc::new(crate::EventState::new());

        let sender_event = UiEventState {
            sender: self.channel.0.clone(),
            theme: self.preferences.take().theme.clone(),
            preferences: self.preferences.borrow().clone(),
        };
        let mut widgets_builder = WidgetsBuilder::new(
            self.window.main.clone(),
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

            let mut hypr_client = HyprlandClient::new(event_state_clone, self.channel.0.clone());
            tokio::spawn(async move {
                hypr_client.run().await;
            });
        }

        let background = gtk::Box::builder().build();
        background.set_hexpand(true);
        background.set_vexpand(true);
        background.set_halign(gtk::Align::Fill);
        background.set_widget_name("background");

        let overlay = gtk::Overlay::builder()
            .hexpand(true)
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .build();

        overlay.set_child(Some(&background));
        overlay.add_overlay(&section_container);
        window.set_child(Some(&overlay));

        let motion_controller_for_normal_window = self.layer_motion_controller();
        let motion_controller_for_hidden_window = self.hidden_bar_motion_controller();

        window.add_controller(motion_controller_for_normal_window.clone());
        hidden_window.add_controller(motion_controller_for_hidden_window.clone());

        if !self.preferences.take().autohide {
            hidden_window.set_focusable(false);
            window.set_focusable(false);
            is_window_visible.set(true);
        }

        let window_clone = window.clone();
        let hidden_window_clone = hidden_window.clone();
        let is_window_visible_clone = Rc::clone(&is_window_visible);

        let is_fullscreen_clone = Rc::clone(&self.window.is_fullscreen);
        let user_config = Rc::clone(&user_config);

        let receiver = self.channel.1.clone();
        let this = Arc::clone(&self);
        glib::MainContext::default().spawn_local(async move {
            while let Ok(msg) = receiver.recv().await {
                match msg {
                    UiEvent::PreferencesChanged(preference) => this.preferences_changed(preference),
                    UiEvent::FullscreenChanged(is_fullscreen) => {
                        handle_fullscreen_visibility(
                            &window_clone,
                            &hidden_window_clone,
                            Rc::clone(&is_window_visible_clone),
                            is_fullscreen,
                        );
                        is_fullscreen_clone.set(is_fullscreen);
                    }
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
                        //widgets_builder.widgets.workspaces.generate_previews();
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

                        //widgets_builder.widgets.workspaces.update_previews();
                    }
                }
            }
        });
        hidden_window.present();
        window.present();
    }

    fn preferences_changed(&self, preference: PreferencesEvent) {
        match preference {
            PreferencesEvent::Reload => {}
            PreferencesEvent::ThemeChanged(theme) => load_css(&theme),
            PreferencesEvent::AutohideChanged(autohide) => {
                if autohide {
                    self.window.hidden.set_focusable(true);
                    self.window.main.hide();
                } else {
                    self.window.hidden.set_focusable(false);
                    self.window.main.set_focusable(false);
                    self.window.main.present();
                }
                self.preferences.borrow_mut().autohide = autohide;
            }
        }
    }

    pub fn layer_motion_controller(&self) -> gtk::EventControllerMotion {
        let motion_controller = gtk::EventControllerMotion::new();

        let bar_clone = self.window.main.clone();
        let hidden_bar_clone_for_leave = self.window.hidden.clone();
        let is_fullscreen = Rc::clone(&self.window.is_fullscreen);
        let preferences = Rc::clone(&self.preferences);
        let is_visible = Rc::clone(&self.window.is_visible);

        motion_controller.connect_leave(move |_| {
            let preferences = preferences.borrow();

            if !is_fullscreen.get() && !preferences.autohide {
                return;
            }

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

    pub fn hidden_bar_motion_controller(&self) -> gtk::EventControllerMotion {
        let motion_controller = gtk::EventControllerMotion::new();
        let hidden_bar_clone = self.window.hidden.clone();
        let bar_clone = self.window.main.clone();

        let is_fullscreen = Rc::clone(&self.window.is_fullscreen);
        let preferences = Rc::clone(&self.preferences);
        let is_visible = Rc::clone(&self.window.is_visible);

        motion_controller.connect_enter(move |_, _x, _y| {
            let preferences = preferences.borrow();

            if !is_fullscreen.get() && !preferences.autohide {
                return;
            }

            is_visible.set(true);
            hidden_bar_clone.set_focusable(false);
            bar_clone.present();
            bar_clone.set_focusable(true);
        });
        motion_controller
    }
}

pub fn set_popover(button: &gtk::Button, child: gtk::Widget) {
    let popover = gtk::Popover::builder()
        .child(&child)
        .has_arrow(true)
        .autohide(true)
        .position(gtk::PositionType::Bottom)
        .build();

    popover.add_css_class("popover");
    popover.set_parent(button);

    let popover_show = popover.clone();

    button.connect_clicked(move |_| {
        popover_show.popup();
    });
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
