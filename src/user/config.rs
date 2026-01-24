use crate::user::models::UserConfig;
use std::fs;

pub fn load_config() -> Option<UserConfig> {
    let path = dirs_next::config_dir()?.join("muelle").join("config.json");
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).expect("Error al parsear el archivo de configuración del usuario")
}
