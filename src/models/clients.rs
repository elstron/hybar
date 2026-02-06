use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

pub type Clients = Vec<Client>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub address: String,
    pub mapped: bool,
    pub hidden: bool,
    pub at: Vec<i64>,
    pub size: Vec<i64>,
    pub workspace: Workspace,
    pub floating: bool,
    pub pseudo: bool,
    pub monitor: i64,
    pub class: String,
    pub title: String,
    pub initial_class: String,
    pub initial_title: String,
    pub pid: i64,
    pub xwayland: bool,
    pub pinned: bool,
    pub fullscreen: i64,
    pub fullscreen_client: i64,
    pub grouped: Vec<Value>,
    pub tags: Vec<Value>,
    pub swallowing: String,
    #[serde(rename = "focusHistoryID")]
    pub focus_history_id: i64,
    pub inhibiting_idle: bool,
    pub xdg_tag: String,
    pub xdg_description: String,
    pub content_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: i64,
    pub name: String,
}
