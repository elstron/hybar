use crate::utils::desktop_dirs;
use std::fs;
#[derive(Debug)]
pub struct DesktopFile {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
}

pub fn search_desktop_file(file_name: &str) -> Option<DesktopFile> {
    let has_find = command_exists("find");
    let has_grep = command_exists("grep");

    if !has_find || !has_grep {
        return search_file(file_name);
    }

    let mut desktop_info = None;

    let path = std::process::Command::new("find")
        .args(desktop_dirs())
        .args([
            "-name",
            "*.desktop",
            "-exec",
            "grep",
            "-il",
            "--",
            file_name,
            "{}",
            "+",
        ])
        .output()
        .expect("Failed to execute command");

    let output = String::from_utf8_lossy(&path.stdout);
    for line in output.lines() {
        let contains_filename = line.to_lowercase().contains(&file_name.to_lowercase());

        if !contains_filename {
            continue;
        }

        let Ok(content) = fs::read_to_string(line) else {
            continue;
        };
        desktop_info = parse_desktop_file(&content);
    }
    desktop_info
}

pub fn search_file(file_name: &str) -> Option<DesktopFile> {
    use std::fs;

    let mut app_found = None;
    for dir in desktop_dirs() {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path().to_string_lossy().to_lowercase();
            let file_name = file_name.trim().to_lowercase().replace(' ', "-");

            if !path.contains(&file_name) {
                println!("Skipping file: {}", entry.path().to_string_lossy());
                continue;
            }

            let content = fs::read_to_string(entry.path()).ok();
            println!("content: {:?}", content);
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

fn command_exists(cmd: &str) -> bool {
    if let Some(paths) = std::env::var_os("PATH") {
        for path in std::env::split_paths(&paths) {
            if path.join(cmd).is_file() {
                return true;
            }
        }
    }
    false
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_file_from_dir_if_exists() {
        let expected_name = "Visual Studio Code".to_string();
        let result = search_desktop_file("code");

        let app_name = if let Some(app) = &result {
            app.name.clone()
        } else {
            "".to_string()
        };

        assert_eq!(app_name, expected_name);
        assert!(result.is_some());
    }

    #[test]
    fn shuld_can_return_some_for_flatpak_app() {
        let expected_name = "Discord".to_string();
        let result = search_desktop_file("discord");

        let app_name = if let Some(app) = &result {
            app.name.clone()
        } else {
            "".to_string()
        };

        assert_eq!(app_name, expected_name);
        assert!(result.is_some());
    }

    #[test]
    fn should_return_none_if_app_not_exist() {
        let result = search_desktop_file("this-app-does-not-exist");
        assert!(result.is_none());
    }
}
