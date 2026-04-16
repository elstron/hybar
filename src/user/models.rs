use serde::Deserialize;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct UserConfig {
    pub theme: String,
    pub ui: UiConfig,
    pub bar: BarConfig,
    pub sections: SectionsConfig,
    pub widgets: HashMap<String, WidgetsConfig>,
    pub custom_apps: HashMap<String, CustomAppsConfig>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct UiConfig {
    pub background: String,
    pub accent: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct BarConfig {
    pub height: u32,
    pub autohide: bool,
    pub position: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct SectionsConfig {
    pub left: Vec<String>,
    pub center: Vec<String>,
    pub right: Vec<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Default, Clone)]
pub struct WidgetsConfig {
    pub favorites: Option<Vec<String>>,
    pub icon: Option<String>,
    pub size: Option<u32>,
    pub format: Option<String>,
    pub timezone: Option<String>,
    pub show_icons: Option<bool>,
    pub max_workspaces: Option<u32>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct CustomAppsConfig {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub cmd: Option<String>,
    pub tooltip: Option<bool>,
}
