use std::fs;
use std::path::PathBuf;

use crate::models::AppConfig;

/// Get the config file path (next to the executable or in app data dir)
pub fn config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("conf.yaml")
}

/// Load config from YAML file, returning defaults if file doesn't exist
pub fn load_config() -> AppConfig {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
        Err(_) => {
            let config = AppConfig::default();
            save_config(&config).ok();
            config
        }
    }
}

/// Save config to YAML file
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    let yaml = serde_yaml::to_string(config).map_err(|e| e.to_string())?;
    fs::write(&path, yaml).map_err(|e| format!("Failed to write config: {}", e))
}
