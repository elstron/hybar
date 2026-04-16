pub mod bootstrap;

use gtk::ApplicationWindow;
use gtk::prelude::WidgetExt;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::enums::preferences::{BarLayout, BarPosition};

pub fn layer_shell_configure(window: &ApplicationWindow, position: &str) {
    LayerShell::init_layer_shell(window);
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("hybar:main"));

    window.auto_exclusive_zone_enable();

    set_position(
        window,
        position.parse::<BarPosition>().unwrap_or(BarPosition::Top),
    );

    window.set_visible(false);
    window.add_css_class("top-bar");
}

pub fn hidden_layer_configuration(window: &ApplicationWindow, position: &str) {
    LayerShell::init_layer_shell(window);
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_focusable(true);
    window.auto_exclusive_zone_enable();
    window.set_namespace(Some("hybar:hidden"));

    set_position(
        window,
        position.parse::<BarPosition>().unwrap_or(BarPosition::Top),
    );

    window.set_visible(false);
    window.add_css_class("hidden-bar");
}
#[allow(dead_code)]
pub fn set_layout(window: &ApplicationWindow, layout: BarLayout) {
    match layout {
        BarLayout::FullWidth => {
            window.set_halign(gtk::Align::Fill);
        }
        BarLayout::Centered => {
            window.set_halign(gtk::Align::Center);
        }
        BarLayout::Floating => {
            window.set_halign(gtk::Align::Center);
            window.set_margin_start(20);
            window.set_margin_end(20);
        }
    }
}

pub fn set_position(window: &ApplicationWindow, position: BarPosition) {
    println!("Setting bar position to {:?}", position);
    match position {
        BarPosition::Top => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Bottom, false);
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Right, true);
        }
        BarPosition::Bottom => {
            window.set_anchor(Edge::Top, false);
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Right, true);
        }
        BarPosition::Left => {
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Right, false);
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Bottom, true);
        }
        BarPosition::Right => {
            window.set_anchor(Edge::Right, true);
            window.set_anchor(Edge::Left, false);
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Bottom, true);
        }
    }
}
