#[derive(Debug)]
pub struct DesktopFile {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
}

pub fn search_desktop_file(file_name: &str) -> Option<DesktopFile> {
    use std::fs;
    //use std::path::Path;
    let shared_dir = &format!(
        "{}/.local/share/applications",
        std::env::var("HOME").unwrap()
    );
    let desktop_dirs = vec![
        "/usr/share/applications",
        "/usr/local/share/applications",
        "/var/lib/flatpak/exports/share/applications/",
        shared_dir,
    ];

    let mut app_found = None;
    for dir in desktop_dirs {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path().to_string_lossy().to_lowercase();
            let mut file_name = file_name.trim().to_lowercase().replace(' ', "-");

            file_name = file_name
                .split('.')
                .nth(2)
                .unwrap_or(&file_name)
                .to_string();

            if !path.contains(&file_name) || !entry.path().to_string_lossy().contains(&file_name) {
                continue;
            }

            let content = fs::read_to_string(entry.path()).ok();
            app_found = content.and_then(|c| parse_desktop_file(&c));
            break;
        }
    }
    app_found
}

fn parse_desktop_file(content: &str) -> Option<DesktopFile> {
    let mut name = None;
    let mut exec = None;
    let mut icon = None;

    for line in content.lines() {
        if name.is_none()
            && let Some(stripped) = line.strip_prefix("Name=")
        {
            name = Some(stripped.to_string());
        }

        if exec.is_none()
            && let Some(stripped) = line.strip_prefix("Exec=")
        {
            let exec_clean = stripped
                .split_whitespace()
                .filter(|s| !s.starts_with('%'))
                .collect::<Vec<_>>()
                .join(" ");
            exec = Some(exec_clean);
        }
        if icon.is_none()
            && let Some(stripped) = line.strip_prefix("Icon=")
        {
            icon = Some(stripped.to_string());
        }

        if name.is_some() && exec.is_some() && icon.is_some() {
            break;
        }
    }

    Some(DesktopFile {
        name: name?,
        exec: exec?,
        icon,
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_file_from_dir_if_exists() {
        let result = search_desktop_file("code");
        assert!(result.is_some());
    }

    #[test]
    fn get_file_from_dir_if_not_exists() {
        let result = search_desktop_file("this-app-does-not-exist");
        assert!(result.is_none());
    }
}
