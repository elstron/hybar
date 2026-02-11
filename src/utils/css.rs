use directories::ProjectDirs;
use gtk::CssProvider;
use gtk::gdk::Display;
use std::fs;

pub fn load_css(theme: &str) {
    let provider = CssProvider::new();

    let fallback_css = include_str!("../style.css");

    let css_data = get_css_from_config(theme).unwrap_or_else(|| fallback_css.to_string());

    provider.load_from_data(&css_data);

    if let Some(display) = Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );
    }
}

fn get_css_from_config(theme: &str) -> Option<String> {
    let proj_dirs = ProjectDirs::from("com", "stron", "hybar")?;
    let css_path = proj_dirs.config_dir().join(format!("themes/{}.css", theme));

    fs::read_to_string(css_path).ok()
}
