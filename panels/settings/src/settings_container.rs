use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, graphene};

pub mod imp {
    use super::*;

    #[derive(Default)]
    pub struct SettingsContainer;

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsContainer {
        const NAME: &'static str = "SettingsContainer";
        type Type = super::SettingsContainer;
        type ParentType = gtk::Box;
    }

    impl ObjectImpl for SettingsContainer {}
    impl WidgetImpl for SettingsContainer {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let width = widget.width() as f64;
            let height = widget.height() as f64;

            let rect = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);

            let cr = snapshot.append_cairo(&rect);

            let r = 40.0;

            let top_radius: f64 = 15.0;
            let bottom_radius: f64 = 25.0;
            let concave_radius: f64 = 35.0;

            let top_height = 60.0; // grosor barra superior
            let stem_width = 220.0; // ancho parte vertical
            let center_x = width / 2.0;

            let stem_left = center_x - stem_width / 2.0;
            let stem_right = center_x + stem_width / 2.0;

            cr.set_source_rgba(0.85, 0.65, 0.65, 1.0);

            // ─── START TOP LEFT ───
            cr.move_to(top_radius, 0.0);

            // Top line
            cr.line_to(width - top_radius, 0.0);

            // Top-right normal radius
            cr.arc(
                width - top_radius,
                top_radius,
                top_radius,
                -90f64.to_radians(),
                0f64.to_radians(),
            );

            cr.line_to(width, top_height);

            // Line to right concave start
            cr.line_to(stem_right + concave_radius, top_height);

            // 🔥 Right concave curve
            cr.arc(
                stem_right,
                top_height + concave_radius,
                concave_radius,
                -90f64.to_radians(),
                -180f64.to_radians(),
            );

            // Down right side
            cr.line_to(stem_right, height - bottom_radius);

            // Bottom-right normal radius
            cr.arc(
                stem_right - bottom_radius,
                height - bottom_radius,
                bottom_radius,
                0f64.to_radians(),
                90f64.to_radians(),
            );

            // Bottom line
            cr.line_to(stem_left + bottom_radius, height);

            // Bottom-left normal radius
            cr.arc(
                stem_left + bottom_radius,
                height - bottom_radius,
                bottom_radius,
                90f64.to_radians(),
                180f64.to_radians(),
            );

            // Up left side
            cr.line_to(stem_left, top_height + concave_radius);

            // 🔥 Left concave curve
            cr.arc(
                stem_left,
                top_height + concave_radius,
                concave_radius,
                180f64.to_radians(),
                270f64.to_radians(),
            );

            cr.line_to(0.0, top_height);

            // Top-left normal radius
            cr.arc(
                top_radius,
                top_radius,
                top_radius,
                180f64.to_radians(),
                270f64.to_radians(),
            );

            cr.close_path();

            cr.fill().unwrap();

            // 🔥 Clip para que los hijos respeten la forma
            cr.clip();

            self.parent_snapshot(snapshot);
        }
    }

    impl BoxImpl for SettingsContainer {}
}

glib::wrapper! {
    pub struct SettingsContainer(ObjectSubclass<imp::SettingsContainer>)
        @extends gtk::Widget, gtk::Box;
}

impl SettingsContainer {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .build()
    }
}
