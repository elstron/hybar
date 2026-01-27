use crate::user::models::UserConfig;
use std::fs;

pub fn load_config() -> Option<UserConfig> {
    let config_path = dirs_next::config_dir().map(|dir| dir.join("muelle").join("config.json"));
    let project_path = std::env::current_dir()
        .ok()
        .map(|dir| dir.join("config.json"));

    let raw = config_path
        .and_then(|path| fs::read_to_string(path).ok())
        .or_else(|| project_path.and_then(|path| fs::read_to_string(path).ok()))?;

    serde_json::from_str(&raw).ok()
}
