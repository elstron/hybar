use gtk::gdk::{Cursor, Texture};
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::{Box as GtkBox, GestureClick, Label};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

const ANY_BUTTON: u32 = 0;
const URGENT_CLASS: &str = "workspace-urgent";
const ACTIVE_CLASS: &str = "workspace-active";
const DEFAULT_CLASS: &str = "workspace";
const DEFAULT_ICON: &str = "\u{f111}";

#[derive(Debug, Deserialize, Clone)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    pub lastwindow: String,
}

#[derive(Debug, Clone)]
pub struct WorkspacesWidget {
    root: GtkBox,
    previews: HashMap<i32, Preview>,
    workspaces_cache: HashMap<i32, Workspace>,
}

#[derive(Debug, Clone)]
pub struct Preview {
    pub workspace_id: i32,
    pub texture: gtk::gdk::Texture,
}

impl WorkspacesWidget {
    pub fn new() -> Self {
        let root = GtkBox::new(gtk::Orientation::Horizontal, 5);
        root.add_css_class("workspaces-box");
        let mut workspaces_cache = HashMap::new();

        update_workspaces(&root, None, Some(&HashMap::new()), &mut workspaces_cache);
        Self {
            root,
            previews: HashMap::new(),
            workspaces_cache,
        }
    }

    pub fn widget(&self) -> &GtkBox {
        let _ = &self.root.set_widget_name("workspaces");
        &self.root
    }

    pub fn update(&mut self, urgent: Option<String>) {
        match urgent {
            Some(id) => {
                update_workspaces(
                    &self.root,
                    Some(&id),
                    Some(&self.previews),
                    &mut self.workspaces_cache,
                );
            }
            None => {
                update_workspaces(
                    &self.root,
                    None,
                    Some(&self.previews),
                    &mut self.workspaces_cache,
                );
            }
        }
    }

    pub fn update_previews(&mut self) {
        println!("Actualizando previews...");
        let previews = self.generate_previews();
        for preview in previews {
            self.previews.insert(preview.workspace_id, preview);
        }
    }

    pub fn generate_previews(&self) -> Vec<Preview> {
        let active_ws = get_active_workspace();
        let workspace_prev = Command::new("grim")
            .arg("-")
            .output()
            .expect("Failed to execute grim");

        if workspace_prev.status.success() {
            let bytes = glib::Bytes::from(&workspace_prev.stdout);

            let stream = gtk::gio::MemoryInputStream::from_bytes(&bytes);

            if let Ok(pixbuf_original) = Pixbuf::from_stream(&stream, gtk::gio::Cancellable::NONE)
                && let Some(pixbuf_escalado) =
                    pixbuf_original.scale_simple(150, 90, gtk::gdk_pixbuf::InterpType::Bilinear)
            {
                let texture = Texture::for_pixbuf(&pixbuf_escalado);

                if let Some(active) = active_ws {
                    return vec![Preview {
                        workspace_id: active.id,
                        texture,
                    }];
                }
            }
        } else {
            eprintln!(
                "Error al tomar captura con grim: {:?}",
                workspace_prev.stderr
            );
        }

        vec![] //
    }
}

pub fn get_workspaces() -> Vec<Workspace> {
    let output = Command::new("hyprctl")
        .arg("workspaces")
        .arg("-j")
        .output()
        .expect("Error ejecutando hyprctl");

    let json_str = String::from_utf8_lossy(&output.stdout);

    serde_json::from_str(&json_str).expect("Error al parsear JSON")
}

pub fn update_workspaces(
    container: &GtkBox,
    urgent_id: Option<&String>,
    textures: Option<&HashMap<i32, Preview>>,
    ws_cache: &mut HashMap<i32, Workspace>,
) {
    let active_ws = get_active_workspace();
    let mut workspaces = get_workspaces();

    workspaces.sort_by_key(|ws| ws.id);
    hide_workspaces(container);

    let cursor = Cursor::from_name("pointer", None);
    for ws in workspaces {
        let label = match child_exists(ws_cache, container, &ws.id)
            .and_then(|w| w.downcast::<Label>().ok())
        {
            Some(l) => l,
            None => {
                let label = Label::new(Some(DEFAULT_ICON));

                ws_cache.insert(ws.id, ws.clone());
                container.append(&label);
                workspace_gesture(&label, ws.name.clone());

                label.set_widget_name(&format!("workspace-{}", ws.id));
                label.set_tooltip_text(Some(&format!("Workspace {}", ws.name)));
                label.set_cursor(cursor.as_ref());
                label
            }
        };
        label.remove_css_class(URGENT_CLASS);
        label.remove_css_class(ACTIVE_CLASS);

        let class = match (active_ws.as_ref(), urgent_id) {
            (Some(active), _) if ws.id == active.id => ACTIVE_CLASS,
            (_, Some(urgent)) if ws.lastwindow.contains(urgent) => URGENT_CLASS,
            _ => DEFAULT_CLASS,
        };

        label.add_css_class(class);

        //let select_texture = textures.and_then(|t| t.get(&ws.id)).cloned();
        //if let Some(texture) = select_texture {
    }
}

fn get_active_workspace() -> Option<Workspace> {
    let output = std::process::Command::new("hyprctl")
        .args(["activeworkspace", "-j"])
        .output()
        .expect("failed to execute hyprctl activeworkspace");
    if output.status.success() {
        serde_json::from_slice::<Workspace>(&output.stdout).ok()
    } else {
        None
    }
}

fn child_exists(
    workspaces: &HashMap<i32, Workspace>,
    parent: &GtkBox,
    id: &i32,
) -> Option<gtk::Widget> {
    let worspace_exists = workspaces.get(id).is_some();

    if !worspace_exists {
        return None;
    }

    let box_ = parent;
    let mut child = box_.first_child();
    while let Some(c) = child {
        child = c.next_sibling();
        if c.widget_name() == format!("workspace-{}", id) {
            c.show();
            return Some(c);
        } else {
            continue;
        }
    }
    None
}

fn hide_workspaces(container: &GtkBox) {
    let box_ = container;
    let mut child = box_.first_child();
    while let Some(c) = child {
        child = c.next_sibling();
        c.hide();
    }
}

fn workspace_gesture(label: &Label, ws_name: String) {
    let gesture = GestureClick::new();
    gesture.set_button(ANY_BUTTON);
    gesture.connect_pressed(move |_, _, _, _| {
        let _ = Command::new("hyprctl")
            .args(["dispatch", "workspace", &ws_name])
            .output();
    });
    label.add_controller(gesture);
}
