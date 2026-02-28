use gtk::{Box as GtkBox, Orientation, prelude::*};

#[derive(Debug, Clone)]
pub struct BarSections {
    pub left: GtkBox,
    pub right: GtkBox,
    pub center: GtkBox,
    pub container: GtkBox,
}

pub fn create_sections() -> BarSections {
    let section_left = gtk::Box::new(Orientation::Horizontal, 0);
    section_left.set_halign(gtk::Align::Start);
    section_left.set_hexpand(true);
    section_left.add_css_class("section-left");
    section_left.set_widget_name("section-left");

    let section_right = gtk::Box::new(Orientation::Horizontal, 0);
    section_right.set_halign(gtk::Align::End);
    section_right.set_hexpand(true);
    section_right.add_css_class("section-right");
    section_right.set_widget_name("section-right");

    let section_center = gtk::Box::new(Orientation::Horizontal, 0);
    section_center.set_halign(gtk::Align::Center);
    section_center.set_hexpand(true);
    section_center.add_css_class("section-center");
    section_center.set_widget_name("section-center");

    let section_container = gtk::Box::new(Orientation::Horizontal, 5);
    section_container.add_css_class("section-container");
    section_container.set_homogeneous(true);
    section_container.set_vexpand(false);
    section_container.set_hexpand(false);
    section_container.set_halign(gtk::Align::Fill);
    section_container.set_valign(gtk::Align::Fill);
    section_container.append(&section_left);
    section_container.append(&section_center);
    section_container.append(&section_right);

    BarSections {
        left: section_left,
        right: section_right,
        center: section_center,
        container: section_container,
    }
}
