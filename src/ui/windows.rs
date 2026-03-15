use gtk::{Application, ApplicationWindow, prelude::*};
use gtk4_layer_shell::LayerShell;
use std::{cell::Cell, rc::Rc};

use crate::config::{BarPosition, hidden_layer_configuration, layer_shell_configure, set_position};

pub struct BarWindows {
    pub is_visible: Rc<Cell<bool>>,
    pub is_fullscreen: Rc<Cell<bool>>,
    pub main: ApplicationWindow,
    pub hidden: ApplicationWindow,
}

impl BarWindows {
    pub fn new(app: &Application) -> Self {
        let bar = Self {
            main: ApplicationWindow::new(app),
            hidden: ApplicationWindow::new(app),
            is_visible: Rc::new(Cell::new(true)),
            is_fullscreen: Rc::new(Cell::new(false)),
        };

        bar.main_window_settings();
        bar.hidden_window_settings();
        bar
    }

    fn main_window_settings(&self) {
        self.main.set_title(Some("hybar"));
        self.main.set_default_height(40);
        LayerShell::init_layer_shell(&self.main);

        layer_shell_configure(&self.main, "top");
    }

    fn hidden_window_settings(&self) {
        self.hidden.set_title(Some("hidden hybar"));
        self.hidden.set_default_height(2);
        LayerShell::init_layer_shell(&self.hidden);

        hidden_layer_configuration(&self.hidden, "top");
    }

    pub fn handle_fullscreen(&self, is_window_visible: Rc<Cell<bool>>, is_fullscreen: bool) {
        if is_fullscreen {
            self.main.hide();
            self.hidden.set_focusable(true);
            is_window_visible.set(false);
        } else {
            self.hidden.set_focusable(false);
            self.main.present();
            is_window_visible.set(true);
        }

        self.is_fullscreen.set(is_fullscreen);
    }

    pub fn toggle_autohide(&self, enable: bool) {
        if enable {
            self.hidden.set_focusable(true);
        } else {
            self.hidden.set_focusable(false);
            self.main.set_focusable(false);
            self.main.present();
        }
    }

    pub fn set_bar_position(&self, position: &str) {
        set_position(
            &self.main,
            position.parse::<BarPosition>().unwrap_or(BarPosition::Top),
        );
        set_position(
            &self.hidden,
            position.parse::<BarPosition>().unwrap_or(BarPosition::Top),
        );
    }
}
