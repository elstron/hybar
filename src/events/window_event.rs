use crate::hybar::{Hybar, find_child_by_name_or_id};
use gtk::prelude::*;

impl Hybar {
    pub fn window_opened(&self, id: &str, name: &str) {
        let mut widgets_builder = self.widgets.borrow_mut();
        widgets_builder.update_active_clients();
        let parent = &widgets_builder.widgets.apps;
        let widget = find_child_by_name_or_id(parent, name, id);

        match widget {
            Some(w) => w.add_css_class("opened"),
            None => widgets_builder.create_widget_app(name, id, true),
        }

        let _ = std::time::Duration::from_secs(3);
        widgets_builder.widgets.workspaces.update_previews();
    }

    pub fn window_closed(&self, id: &str) {
        {
            let widgets_builder = self.widgets.borrow();
            widgets_builder.update_active_clients();

            let apps = &widgets_builder.widgets.apps;

            let widget = find_child_by_name_or_id(apps, "", id);
            if let Some(widget) = widget {
                let formatted_id = format!("_{}", id);
                let widget_name = &widget.widget_name().replace(formatted_id.as_str(), "");
                println!("Window closed event for widget: {}", widget_name);
                widget.set_widget_name(widget_name);

                if !widget_name.contains("_") {
                    widget.remove_css_class("opened");

                    let is_favorite = self
                        .preferences
                        .borrow()
                        .favorites
                        .iter()
                        .any(|fav| fav == widget_name);

                    if !is_favorite {
                        let widgets_builder = self.widgets.borrow();
                        widgets_builder.remove_widget_app(&widget);
                    }
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
