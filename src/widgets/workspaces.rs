use gtk::prelude::*;
use gtk::{Box as GtkBox, GestureClick, Label};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    pub lastwindow: String,
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

pub fn update_workspaces(container: &GtkBox, urgent_id: Option<&String>) {
    let active_ws = get_active_workspace();

    {
        let box_ = container;
        let mut child = box_.first_child();
        while let Some(c) = child {
            child = c.next_sibling();
            box_.remove(&c);
        }
    }

    let mut workspaces = get_workspaces();

    workspaces.sort_by_key(|ws| ws.id);

    for ws in workspaces {
        let label = Label::new(None);
        label.set_text("\u{f111}");

        if let Some(active) = &active_ws {
            if ws.id == active.id {
                label.add_css_class("workspace-active");
                label.set_text("\u{f192}");
            } else if let Some(urgent) = urgent_id
                && ws.lastwindow.contains(urgent)
            {
                label.add_css_class("workspace-urgent");
                label.set_text("\u{f111}");
            } else {
                label.add_css_class("workspace");
                label.set_text("\u{f111}");
            }
        } else {
            label.add_css_class("workspace");
            label.set_text("\u{f111}");
        }

        let gesture = GestureClick::new();
        gesture.set_button(0); // acepta cualquier botón

        let ws_name = ws.name.clone();
        gesture.connect_pressed(move |_, _, _, _| {
            let _ = Command::new("hyprctl")
                .args(["dispatch", "workspace", &ws_name])
                .output();
        });

        label.add_controller(gesture);
        container.append(&label);
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
