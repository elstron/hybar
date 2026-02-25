use gtk::{Orientation, pango, prelude::*};

#[derive(Debug, Clone)]
pub struct TitleWidget {
    root: gtk::Widget,
    title_label: gtk::Label,
}

impl TitleWidget {
    pub fn new() -> Self {
        let title_container = gtk::Box::new(Orientation::Horizontal, 5);
        title_container.add_css_class("title-container");

        let title_label = gtk::Label::new(Some(""));
        title_label.set_ellipsize(pango::EllipsizeMode::End);
        title_label.set_max_width_chars(100);
        title_container.append(&title_label);

        Self {
            root: title_container.into(),
            title_label,
        }
    }

    pub fn widget(&self) -> &gtk::Widget {
        &self.root
    }

    pub fn set_title(&self, title: &str) {
        self.title_label.set_text(title);
    }
}
