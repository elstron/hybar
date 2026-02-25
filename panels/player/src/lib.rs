use gtk::gdk::Texture;
use gtk::glib;
use gtk::prelude::*;
use gtk::ApplicationWindow;
use gtk::{Align, Box, Button, Label, Orientation};
use gtk4_layer_shell::LayerShell;
use mpris::{PlaybackStatus, PlayerFinder};
use std::time::Duration;

pub fn build_ui() -> (ApplicationWindow, Label) {
    let main_box = main_box();
    let activity_container = activity_container();
    let label = activity_label();
    let status_label = status_label();
    let cover_art = cover_art_image(100);

    let scrolled_window = title_label_container(&label);

    let (hbox_buttons, btn_play, btn_next, btn_prev) = actions_box();

    main_box.append(&cover_art);
    main_box.append(&activity_container);
    activity_container.append(&status_label);
    activity_container.append(&scrolled_window);
    activity_container.append(&hbox_buttons);

    let window = window(main_box);

    let btn_play_clone = btn_play.clone();

    btn_play.connect_clicked(move |_| {
        let btn = &btn_play_clone;
        let play_icon = "media-playback-start-symbolic";
        let pause_icon = "media-playback-pause-symbolic";

        let is_playing = btn.icon_name().as_deref() == Some(play_icon);

        match is_playing {
            true => btn.set_icon_name(pause_icon),
            false => btn.set_icon_name(play_icon),
        };

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
    let cover_art_clone = cover_art.clone();

    glib::timeout_add_local(Duration::from_millis(500), move || {
        update_status(&label_clone, &status_clone, &cover_art_clone);
        glib::ControlFlow::Continue
    });

    (window, label)
}

pub fn window(child: gtk::Box) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .title("Settings")
        .default_width(400)
        .default_height(150)
        .build();
    LayerShell::init_layer_shell(&window);

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

fn activity_label() -> Label {
    let label = Label::new(Some("💤 No activity"));
    label.add_css_class("song-label");
    label.set_wrap(false);
    label
}

fn status_label() -> Label {
    let label = Label::new(Some("Scanning for players..."));
    label.add_css_class("status-label");
    label.set_wrap(false);
    label
}

fn cover_art_image(size: i32) -> gtk::Picture {
    let cover_art = gtk::Picture::new();
    cover_art.set_width_request(size);
    cover_art.set_valign(Align::Start);
    cover_art.add_css_class("cover-art");
    cover_art
}

fn activity_container() -> Box {
    let hbox = Box::new(Orientation::Vertical, 10);
    hbox.set_halign(Align::Start);
    hbox
}

fn title_label_container(label: &Label) -> gtk::ScrolledWindow {
    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_min_content_width(300);
    scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
    scrolled_window.set_child(Some(label));

    let adjustment = scrolled_window.vadjustment();
    adjustment.set_value(adjustment.value() + 1.0);
    scrolled_window
}

fn main_box() -> Box {
    let vbox = Box::new(Orientation::Horizontal, 10);
    vbox.set_valign(Align::Center);
    vbox
}

fn actions_box() -> (Box, Button, Button, Button) {
    let hbox_buttons = Box::new(Orientation::Horizontal, 10);
    hbox_buttons.set_halign(Align::Center);

    let btn_play = Button::from_icon_name("media-playback-start-symbolic");
    let btn_next = Button::from_icon_name("media-skip-forward-symbolic");
    let btn_prev = Button::from_icon_name("media-skip-backward-symbolic");

    hbox_buttons.append(&btn_prev);
    hbox_buttons.append(&btn_play);
    hbox_buttons.append(&btn_next);

    (hbox_buttons, btn_play, btn_next, btn_prev)
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

fn update_status(label: &Label, status: &Label, cover_art: &gtk::Picture) {
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

        let status_icon = match player
            .get_playback_status()
            .unwrap_or(PlaybackStatus::Stopped)
        {
            PlaybackStatus::Playing => "🎵 Playing",
            PlaybackStatus::Paused => "⏸ Paused",
            PlaybackStatus::Stopped => "⏹ Stopped",
        };

        if label.text().contains(title) && status.text() == status_icon {
            return;
        }

        let artists = metadata
            .as_ref()
            .and_then(|m| m.artists())
            .map(|a| a.join(", "))
            .unwrap_or_default();

        let c_art = metadata.as_ref().and_then(|m| m.art_url());

        let text = format!("{} - {}", title, artists);
        label.set_text(&text);
        status.set_text(status_icon);

        let file = gtk::gio::File::for_uri(c_art.unwrap_or_default());
        let texture = Texture::from_file(&file).ok();

        cover_art.set_paintable(texture.as_ref());
    } else {
        label.set_text("💤 No activity");
    }
}
