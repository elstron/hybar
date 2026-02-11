use crate::user::models::{SectionsConfig, UserConfig};
use std::{collections::HashMap, fs};

pub fn load_config() -> Option<UserConfig> {
    let config_path = dirs_next::config_dir().map(|dir| dir.join("hybar").join("config.json"));
    let project_path = std::env::current_dir()
        .ok()
        .map(|dir| dir.join("config.json"));

    let raw = config_path
        .and_then(|path| fs::read_to_string(path).ok())
        .or_else(|| project_path.and_then(|path| fs::read_to_string(path).ok()))?;

    let config: Option<UserConfig> = serde_json::from_str(&raw).ok();

    if let Some(config) = config {
        let mut seen: HashMap<String, usize> = HashMap::new();
        Some(UserConfig {
            sections: SectionsConfig {
                left: normalize_duplicate_keys(config.sections.left.clone(), &mut seen),
                center: normalize_duplicate_keys(config.sections.center.clone(), &mut seen),
                right: normalize_duplicate_keys(config.sections.right.clone(), &mut seen),
            },
            ..config
        })
    } else {
        config
    }
}

fn normalize_duplicate_keys(
    keys: Vec<String>,
    seen: &mut std::collections::HashMap<String, usize>,
) -> Vec<String> {
    let mut normalized = Vec::new();

    for key in keys {
        let count = seen.entry(key.clone()).or_insert(0);

        let unique_key = if *count == 0 {
            key.clone()
        } else {
            format!("{}_{}", key, count)
        };

        *count += 1;
        normalized.push(unique_key);
    }

    normalized
}
