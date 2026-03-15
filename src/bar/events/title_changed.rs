use crate::bar::{Hybar, find_child_by_name_or_id};
use gtk::prelude::*;

impl Hybar {
    pub fn title_changed(&self, title: &str) {
        let widgets_builder = self.widgets.borrow();
        widgets_builder.widgets.title.set_title(title);
        let client_name = title.split(",").next().unwrap_or("");
        let widget = find_child_by_name_or_id(&widgets_builder.widgets.apps, client_name, "");
        if let Some(widget) = widget {
            widget.grab_focus();
        }
    }
}
