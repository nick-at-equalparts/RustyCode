use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persisted user configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub theme: Option<String>,
    /// Saved model selection: (provider_id, model_id).
    #[serde(default)]
    pub model: Option<(String, String)>,
}

impl Config {
    /// Load config from disk, returning defaults if the file doesn't exist or is invalid.
    pub fn load() -> Self {
        let path = match config_path() {
            Some(p) => p,
            None => return Self::default(),
        };

        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save the current config to disk.
    pub fn save(&self) {
        let path = match config_path() {
            Some(p) => p,
            None => return,
        };

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }
}

/// Returns `~/.config/rustycode/config.json`, or `None` if the home dir can't be determined.
fn config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("rustycode")
            .join("config.json"),
    )
}
