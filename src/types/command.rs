use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub name: String,
    pub description: Option<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub source: Option<CommandSource>,
    pub template: Option<String>,
    pub subtask: Option<bool>,
    pub hints: Option<Vec<String>>,
    pub shortcut: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandSource {
    #[serde(rename = "command")]
    Command,
    #[serde(rename = "mcp")]
    Mcp,
    #[serde(rename = "skill")]
    Skill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub name: String,
    pub description: Option<String>,
    pub mode: Option<String>,
    pub native: Option<bool>,
    pub hidden: Option<bool>,
    pub color: Option<String>,
    pub model: Option<String>,
    pub tools: Option<Vec<String>>,
    pub system: Option<String>,
    pub options: Option<serde_json::Value>,
    pub permission: Option<serde_json::Value>,
}
