use gtk::glib;
use gtk::prelude::*;
use gtk::ApplicationWindow;
use gtk::{Align, Box, Button, Label, Orientation};
use gtk4_layer_shell::LayerShell;
use mpris::{PlaybackStatus, PlayerFinder};
use std::time::Duration;

pub fn build_ui() -> (ApplicationWindow, Label) {
    let vbox = Box::new(Orientation::Vertical, 10);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(5);
    vbox.set_margin_end(5);
    vbox.set_halign(Align::Center);

    let label = Label::new(Some("💤 No activity"));
    label.add_css_class("song-label");
    label.set_wrap(false);
    let status_label = Label::new(Some("Scanning for players..."));

    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_min_content_width(300);
    scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
    scrolled_window.set_child(Some(&label));

    let adjustment = scrolled_window.vadjustment();
    adjustment.set_value(adjustment.value() + 1.0);

    let hbox_buttons = Box::new(Orientation::Horizontal, 10);
    hbox_buttons.set_halign(Align::Center);

    let btn_play = Button::from_icon_name("media-playback-start-symbolic");
    let btn_next = Button::from_icon_name("media-skip-forward-symbolic");
    let btn_prev = Button::from_icon_name("media-skip-backward-symbolic");

    hbox_buttons.append(&btn_prev);
    hbox_buttons.append(&btn_play);
    hbox_buttons.append(&btn_next);

    vbox.append(&status_label);
    vbox.append(&scrolled_window);
    vbox.append(&hbox_buttons);

    let window = window(vbox);

    let btn_play_clone = btn_play.clone();
    btn_play.connect_clicked(move |_| {
        let btn = &btn_play_clone;
        let play_icon = "media-playback-start-symbolic";
        let pause_icon = "media-playback-pause-symbolic";

        let is_playing = btn.icon_name().as_deref() == Some(play_icon);

        if is_playing {
            btn.set_icon_name(pause_icon);
        } else {
            btn.set_icon_name(play_icon);
        }

        control_media("play_pause");
    });

    btn_next.connect_clicked(move |_| {
        control_media("next");
    });

    btn_prev.connect_clicked(move |_| {
        control_media("previous");
    });

    let label_clone = label.clone();
    let status_clone = status_label.clone();
    glib::timeout_add_local(Duration::from_millis(500), move || {
        update_status(&label_clone, &status_clone);
        glib::ControlFlow::Continue
    });

    (window, label)
}

pub fn window(child: gtk::Box) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .title("Settings")
        .default_width(400)
        .build();
    LayerShell::init_layer_shell(&window);

    window.auto_exclusive_zone_enable();
    window.set_layer(gtk4_layer_shell::Layer::Overlay);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, false);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, false);
    window.set_namespace(Some("hybar:player"));

    window.set_child(Some(&child));
    window.add_css_class("settings-window");

    window
}

fn control_media(command: &str) {
    let finder = match PlayerFinder::new() {
        Ok(f) => f,
        Err(_) => return,
    };

    if let Ok(player) = finder.find_active() {
        match command {
            "play_pause" => {
                let _ = player.play_pause();
            }
            "next" => {
                let _ = player.next();
            }
            "previous" => {
                let _ = player.previous();
            }
            _ => {}
        }
    } else {
        println!("No active media player found");
    }
}

fn update_status(label: &Label, status: &Label) {
    let finder = match PlayerFinder::new() {
        Ok(f) => f,
        Err(_) => {
            label.set_text("Error D-Bus");
            return;
        }
    };

    if let Ok(player) = finder.find_active() {
        let metadata = player.get_metadata().ok();

        let title = metadata
            .as_ref()
            .and_then(|m| m.title())
            .unwrap_or("Unknown");
        let artists = metadata
            .as_ref()
            .and_then(|m| m.artists())
            .map(|a| a.join(", "))
            .unwrap_or_default();

        let status_icon = match player
            .get_playback_status()
            .unwrap_or(PlaybackStatus::Stopped)
        {
            PlaybackStatus::Playing => "🎵 Playing",
            PlaybackStatus::Paused => "⏸ Paused",
            PlaybackStatus::Stopped => "⏹ Stopped",
        };

        let text = format!("{} - {}", title, artists);
        label.set_text(&text);
        status.set_text(status_icon);
    } else {
        label.set_text("💤 No activity");
    }
}
