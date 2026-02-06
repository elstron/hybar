use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct UserConfig {
    pub ui: UiConfig,
    pub bar: BarConfig,
    pub sections: SectionsConfig,
    pub widgets: HashMap<String, WidgetsConfig>,
    pub custom_apps: HashMap<String, CustomAppsConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UiConfig {
    pub background: String,
    pub accent: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct BarConfig {
    pub height: u32,
    pub autohide: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct SectionsConfig {
    pub left: Vec<String>,
    pub center: Vec<String>,
    pub right: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct WidgetsConfig {
    pub favorites: Option<Vec<String>>,
    pub icon: Option<String>,
    pub size: Option<u32>,
    pub format: Option<String>,
    pub timezone: Option<String>,
    pub show_icons: Option<bool>,
    pub max_workspaces: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CustomAppsConfig {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub cmd: Option<String>,
    pub tooltip: Option<bool>,
}
