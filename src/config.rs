pub mod bootstrap;
use std::str::FromStr;

use gtk::ApplicationWindow;
use gtk::prelude::WidgetExt;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

#[derive(Debug)]
pub enum BarPosition {
    Top,
    Bottom,
    Left,
    Right,
}
impl FromStr for BarPosition {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "top" => Ok(BarPosition::Top),
            "bottom" => Ok(BarPosition::Bottom),
            "right" => Ok(BarPosition::Right),
            "left" => Ok(BarPosition::Left),
            _ => Err(()),
        }
    }
}
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
