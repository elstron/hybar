use crate::bar::{Hybar, find_widget_child};
use gtk::prelude::*;

impl Hybar {
    pub fn title_changed(&self, title: &str) {
        let widgets_builder = self.widgets.borrow();
        widgets_builder.widgets.title.set_title(title);

        let client_name = title.split(",").next().unwrap_or("");
        let parent = &widgets_builder.widgets.apps;

        let widget = find_widget_child(parent, client_name);

        if let Some(widget) = widget {
            widget.grab_focus();
        }
    }
}
