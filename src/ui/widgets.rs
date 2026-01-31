pub mod clock;
pub mod separator;
pub mod title;
pub mod workspaces;

use gtk::{Box as GtkBox, gdk::Cursor, gio, prelude::*};
use std::{cell::Cell, collections::HashSet, env, rc::Rc, sync::Arc};

use crate::{EventState, UiEventState, user::models::UserConfig};

pub struct Widgets {
    pub workspaces: workspaces::WorkspacesWidget,
    pub clock: gtk::Widget,
    pub title: title::TitleWidget,
}

pub struct WidgetsBuilder {
    user_config: Rc<UserConfig>,
    event_state: Arc<EventState>,
    is_visible: Rc<Cell<bool>>,
    pub widgets: Widgets,
    widgets_cache: Rc<std::cell::RefCell<std::collections::HashMap<String, gtk::Widget>>>,
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
            },
            widgets_cache: Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())),
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
            "settings" => {
                let settings_button = gtk::Button::with_label("");
                settings_button.add_css_class("settings-button");

                let window = panels::settings::render(self.sender.clone());
                settings_button.connect_clicked(move |_| {
                    println!("Opening settings window");
                    window.present();
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
                            if let Some(app) = gio::DesktopAppInfo::new(&cmd) {
                                let _ = app.launch(&[], None::<&gio::AppLaunchContext>);
                                println!("Launching application: {}", cmd);
                            } else {
                                let _ = std::process::Command::new("sh")
                                    .arg("-c")
                                    .arg(&cmd)
                                    .current_dir(env::var("HOME").unwrap())
                                    .spawn();
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

    pub fn sync_widgets_layout(
        &self,
        section_left: Rc<GtkBox>,
        section_right: Rc<GtkBox>,
        section_center: Rc<GtkBox>,
    ) -> bool {
        let left =
            self.sync_section_widgets(&section_left, self.user_config.sections.left.as_slice());
        let center =
            self.sync_section_widgets(&section_center, self.user_config.sections.center.as_slice());
        let right =
            self.sync_section_widgets(&section_right, self.user_config.sections.right.as_slice());
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
        self.user_config
            .sections
            .left
            .contains(&widget_name.to_string())
            || self
                .user_config
                .sections
                .center
                .contains(&widget_name.to_string())
            || self
                .user_config
                .sections
                .right
                .contains(&widget_name.to_string())
    }

    pub fn update_config(&mut self, user_config: Rc<UserConfig>) {
        self.user_config = user_config;
    }
}
