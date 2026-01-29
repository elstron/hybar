use crate::user::models::UserConfig;
use gtk::prelude::*;

pub fn render(config: &UserConfig) -> gtk::Widget {
    let icon = config.widgets.get("separator").and_then(|w| w.icon.clone());
    let separator = gtk::Label::new(Some(icon.as_deref().unwrap_or("\u{f078}")));
    separator.add_css_class("separator");
    separator.into()
}
