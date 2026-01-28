pub mod clock;
pub mod workspaces;

use glib::ControlFlow;
use gtk::{Box as GtkBox, Orientation, gdk::Cursor, gio, pango, prelude::*};
use std::{cell::Cell, collections::HashSet, env, rc::Rc, sync::Arc, time::Duration};

use crate::{EventState, user::models::UserConfig};

pub fn build_widget(
    name: &str,
    config: &UserConfig,
    event_state: Arc<EventState>,
    is_visible: Rc<Cell<bool>>,
) -> gtk::Widget {
    let name = name.split('_').next().unwrap_or(name);
    match name {
        "separator" => {
            let icon = config.widgets.get("separator").and_then(|w| w.icon.clone());
            let separator = gtk::Label::new(Some(icon.as_deref().unwrap_or("\u{f078}")));
            separator.add_css_class("separator");
            separator.into()
        }
        "workspaces" => workspaces::workspaces_build(Arc::clone(&event_state)),
        "clock" => clock::render(&is_visible),
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
    widgets_cache: &Rc<std::cell::RefCell<std::collections::HashMap<String, gtk::Widget>>>,
    user_config: &UserConfig,
    section_left: Rc<GtkBox>,
    section_right: Rc<GtkBox>,
    section_center: Rc<GtkBox>,
    event_state: &Arc<EventState>,
    is_window_visible: &Rc<Cell<bool>>,
) -> bool {
    let left = sync_section_widgets(
        Rc::clone(widgets_cache),
        &user_config.sections.left,
        &section_left,
        user_config,
        Arc::clone(event_state),
        Rc::clone(is_window_visible),
    );
    let center = sync_section_widgets(
        Rc::clone(widgets_cache),
        &user_config.sections.center,
        &section_center,
        user_config,
        Arc::clone(event_state),
        Rc::clone(is_window_visible),
    );
    let right = sync_section_widgets(
        Rc::clone(widgets_cache),
        &user_config.sections.right,
        &section_right,
        user_config,
        Arc::clone(event_state),
        Rc::clone(is_window_visible),
    );
    left || center || right
}

fn sync_section_widgets(
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
                build_widget(
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

pub fn widget_exists(config: &UserConfig, widget_name: &str) -> bool {
    config.sections.left.contains(&widget_name.to_string())
        || config.sections.center.contains(&widget_name.to_string())
        || config.sections.right.contains(&widget_name.to_string())
}
