pub mod app_launch;
pub mod css;
pub mod search;

pub fn desktop_dirs() -> Vec<String> {
    vec![
        "/usr/share/applications".into(),
        "/usr/local/share/applications".into(),
        "/var/lib/flatpak/exports/share/applications".into(),
        format!(
            "{}/.local/share/flatpak/exports/share/applications",
            std::env::var("HOME").unwrap()
        ),
        format!(
            "{}/.local/share/applications",
            std::env::var("HOME").unwrap()
        ),
    ]
}
