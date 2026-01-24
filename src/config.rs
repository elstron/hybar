use gtk::prelude::WidgetExt;
use gtk::{ApplicationWindow, prelude::StyleContextExt};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

pub fn layer_shell_configure(window: &ApplicationWindow) {
    LayerShell::init_layer_shell(window);
    window.set_layer(Layer::Overlay);
    //window.set_keyboard_mode(KeyboardMode::OnDemand);
    //window.set_focusable(true);
    window.auto_exclusive_zone_enable();
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, false);
    window.set_visible(false);
    window.add_css_class("top-bar");
}

pub fn hidden_layer_configuration(window: &ApplicationWindow) {
    LayerShell::init_layer_shell(window);
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_focusable(true);
    window.auto_exclusive_zone_enable();
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, false);
    window.set_visible(false);
    window.add_css_class("hidden-bar");
}

pub fn set_background_color(window: &ApplicationWindow, bg_color: &str) {
    let css = format!(
        r#"
        .bg_color {{
            background-color: {};
        }}
        "#,
        bg_color
    );

    let provider = gtk::CssProvider::new();
    provider.load_from_data(css.as_str());

    window
        .style_context()
        .add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_USER);
    window.add_css_class("bg_color");
}
