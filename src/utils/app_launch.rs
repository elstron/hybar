use gtk::gio;
use gtk::prelude::*;
use std::env;

pub fn app_lauch(app_cmd: &str) {
    if let Some(app) = gio::DesktopAppInfo::new(app_cmd) {
        let _ = app.launch(&[], None::<&gio::AppLaunchContext>);
    } else {
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(app_cmd)
            .current_dir(env::var("HOME").unwrap())
            .spawn();
    }
}
