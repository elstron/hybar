use crate::bar::{Hybar, find_widget_child};
use gtk::prelude::*;

impl Hybar {
    pub fn window_opened(&self, _id: &str, name: &str) {
        let mut widgets_builder = self.widgets.borrow_mut();
        widgets_builder.update_active_clients();
        let parent = &widgets_builder.widgets.apps;
        let widget = find_widget_child(parent, name);

        match widget {
            Some(w) => w.add_css_class("opened"),
            None => widgets_builder.create_widget_app(name, true),
        }
        widgets_builder.widgets.workspaces.update_previews();
    }

    pub fn window_closed(&self, id: &str) {
        {
            let address = format!("0x{}", id);
            let widgets_builder = self.widgets.borrow();

            let clients = &widgets_builder.get_active_clients();
            let find_client = clients.iter().find(|client| client.address == address);

            widgets_builder.update_active_clients();
            let client = match find_client {
                Some(client) => {
                    let filtered_clients: Vec<_> =
                        clients.iter().filter(|c| c.class == client.class).collect();
                    if filtered_clients.len() > 1 {
                        return;
                    };
                    client
                }
                None => return,
            };
            let apps = &widgets_builder.widgets.apps;

            let widget = find_widget_child(apps, &client.class.to_lowercase());
            if let Some(widget) = widget {
                widget.remove_css_class("opened");
                apps.grab_focus();

                let is_favorite = self
                    .preferences
                    .borrow()
                    .favorites
                    .iter()
                    .any(|fav| fav == &client.class);

                if !is_favorite {
                    let widgets_builder = self.widgets.borrow();
                    widgets_builder.remove_widget_app(&widget);
                }
            }
        }

        self.update_previews();
    }

    fn update_previews(&self) {
        self.widgets
            .borrow_mut()
            .widgets
            .workspaces
            .update_previews();
    }
}
