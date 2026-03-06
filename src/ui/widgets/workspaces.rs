use gtk::gdk::{Cursor, Texture};
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{ApplicationWindow, Box as GtkBox, EventControllerMotion, GestureClick, Label};
use gtk::{EventController, prelude::*};
use gtk4_layer_shell::LayerShell;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::process::Command;
use std::rc::Rc;

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
    preview_window: Rc<RefCell<PreviewWindow>>,
    workspaces_cache: HashMap<i32, Workspace>,
}

#[derive(Debug, Clone)]
pub struct Preview {
    pub id: i32,
    pub texture: gtk::gdk::Texture,
}

impl WorkspacesWidget {
    pub fn new() -> Self {
        let root = GtkBox::new(gtk::Orientation::Horizontal, 5);
        root.add_css_class("workspaces-box");
        let mut workspaces_cache = HashMap::new();
        let preview_window = Rc::new(RefCell::new(PreviewWindow::new()));
        let mut workspacs_w = Self {
            root: root.clone(),
            previews: HashMap::new(),
            preview_window: Rc::clone(&preview_window),
            workspaces_cache: workspaces_cache.clone(),
        };

        update_workspaces(
            &root,
            None,
            &mut workspaces_cache,
            &preview_window.borrow().window,
        );
        workspacs_w.update_previews();
        workspacs_w
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
                    &mut self.workspaces_cache,
                    &self.preview_window.borrow().window,
                );
            }
            None => {
                update_workspaces(
                    &self.root,
                    None,
                    &mut self.workspaces_cache,
                    &self.preview_window.borrow().window,
                );
            }
        }
    }

    pub fn update_previews(&mut self) {
        println!("Actualizando previews...");
        let previews = self.generate_previews();
        self.preview_window.borrow_mut().update(previews);
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
                        id: active.id,
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
    ws_cache: &mut HashMap<i32, Workspace>,
    window: &ApplicationWindow,
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
                workspace_gesture(&label, ws.name.clone(), window);

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

fn workspace_gesture(label: &Label, ws_name: String, window: &ApplicationWindow) {
    let controller = EventControllerMotion::new();

    let ws_clone = ws_name.clone();
    let window_clone = window.clone();
    controller.connect_enter(move |_, _, _| {
        window_clone.show();
        println!("Mouse entered workspace {}", ws_clone);
    });

    let gesture = GestureClick::new();
    gesture.set_button(ANY_BUTTON);
    gesture.connect_pressed(move |_, _, _, _| {
        let _ = Command::new("hyprctl")
            .args(["dispatch", "workspace", &ws_name])
            .output();
    });
    label.add_controller(controller);
    label.add_controller(gesture);
}
#[derive(Debug, Clone)]
pub struct PreviewWindow {
    window: ApplicationWindow,
    main_box: GtkBox,
    previews: HashMap<i32, Preview>,
}
impl PreviewWindow {
    fn new() -> Self {
        let window = ApplicationWindow::builder()
            .title("Preview")
            .default_width(200)
            .default_height(120)
            .build();

        LayerShell::init_layer_shell(&window);

        window.set_layer(gtk4_layer_shell::Layer::Overlay);
        window.set_anchor(gtk4_layer_shell::Edge::Right, false);
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_anchor(gtk4_layer_shell::Edge::Top, true);
        window.set_anchor(gtk4_layer_shell::Edge::Bottom, false);
        window.set_namespace(Some("hybar:preview"));
        window.add_css_class("preview-window");

        let main_box = GtkBox::new(gtk::Orientation::Horizontal, 20);
        main_box.add_css_class("section-left");

        window.set_child(Some(&main_box));

        let controller = EventControllerMotion::new();
        controller.connect_leave({
            let window_clone = window.clone();
            move |_| {
                window_clone.hide();
            }
        });

        window.add_controller(controller);

        Self {
            window,
            main_box,
            previews: HashMap::new(),
        }
    }

    pub fn update(&mut self, previews: Vec<Preview>) {
        self.hide_all_previews();

        for preview in previews {
            self.previews.insert(preview.id, preview);
        }

        for preview in self.previews.clone().values() {
            let picture = match self.child_exists(&preview.id) {
                Some(w) => {
                    let p_box = w.downcast::<gtk::Box>().ok().unwrap();
                    let picture = p_box
                        .first_child()
                        .and_then(|c| c.first_child())
                        .and_then(|c| c.downcast::<gtk::Picture>().ok());

                    if let Some(pic) = picture {
                        pic.set_paintable(Some(&preview.texture));
                    }
                    p_box
                }
                None => self.create_widget_preview(preview),
            };
            picture.show();
        }
    }

    fn hide_all_previews(&mut self) {
        let box_ = self.main_box.clone();
        let mut child = box_.first_child();
        while let Some(c) = child {
            child = c.next_sibling();
            if c.widget_name().starts_with("preview-") {
                c.hide();
            }
        }
    }

    fn child_exists(&mut self, id: &i32) -> Option<gtk::Widget> {
        let pw_exists = self.previews.contains_key(id);

        if !pw_exists {
            return None;
        }

        let box_ = self.main_box.clone();
        let mut child = box_.first_child();
        while let Some(c) = child {
            child = c.next_sibling();
            if c.widget_name() == format!("preview-{}", id) {
                c.show();
                return Some(c);
            } else {
                continue;
            }
        }
        None
    }

    fn add_gesture(&self, button: &gtk::Button, ws_name: String) {
        button.set_cursor(Cursor::from_name("pointer", None).as_ref());
        button.connect_clicked(move |_| {
            let _ = Command::new("hyprctl")
                .args(["dispatch", &ws_name])
                .output();
        });
    }

    fn create_widget_preview(&self, preview: &Preview) -> gtk::Box {
        let ws_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let label = gtk::Label::new(Some(&format!("Workspace {}", preview.id)));
        let button = gtk::Button::new();
        let picture = gtk::Picture::new();

        picture.set_width_request(150);
        picture.set_valign(gtk::Align::Start);
        picture.set_paintable(Some(&preview.texture));

        button.set_child(Some(&picture));

        ws_box.add_css_class("preview");
        ws_box.set_widget_name(&format!("preview-{}", preview.id));
        ws_box.append(&button);
        ws_box.append(&label);

        self.main_box.append(&ws_box);
        self.add_gesture(&button, format!("workspace {}", preview.id));
        ws_box
    }
}
