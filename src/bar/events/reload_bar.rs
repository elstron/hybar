use std::rc::Rc;

use crate::{bar::Hybar, user::config::load_config};

impl Hybar {
    pub fn reload_bar(
        &self,
        section_left: Rc<gtk::Box>,
        section_right: Rc<gtk::Box>,
        section_center: Rc<gtk::Box>,
    ) {
        let mut widgets_builder = self.widgets.borrow_mut();
        let new_config = load_config().unwrap_or_default();
        widgets_builder.update_config(Rc::new(new_config));
        widgets_builder.sync_widgets_layout(
            Rc::clone(&section_left),
            Rc::clone(&section_right),
            Rc::clone(&section_center),
        );
    }
}
