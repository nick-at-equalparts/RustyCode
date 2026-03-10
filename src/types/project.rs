use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: Option<String>,
    pub name: Option<String>,
    pub worktree: Option<String>,
    pub path: Option<String>,
    pub directory: Option<String>,
    pub vcs: Option<serde_json::Value>,
    pub icon: Option<serde_json::Value>,
    pub time: Option<serde_json::Value>,
    pub sandboxes: Option<Vec<serde_json::Value>>,
    pub config: Option<serde_json::Value>,
    pub summary: Option<ProjectSummary>,
    pub paths: Option<ProjectPaths>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub description: Option<String>,
    pub languages: Option<Vec<String>>,
    pub frameworks: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPaths {
    pub root: Option<String>,
    pub config: Option<String>,
    pub data: Option<String>,
    pub state: Option<String>,
    pub cache: Option<String>,
    pub extra: Option<HashMap<String, String>>,
}
