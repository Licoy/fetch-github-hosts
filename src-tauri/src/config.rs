use std::fs;
use std::path::PathBuf;

use crate::models::AppConfig;

/// Get the config directory: ~/.fetch-github-hosts/
fn config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".fetch-github-hosts")
}

/// Get the config file path: ~/.fetch-github-hosts/config.yml
pub fn config_path() -> PathBuf {
    config_dir().join("config.yml")
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
    let dir = config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let path = config_path();
    let yaml = serde_yaml::to_string(config).map_err(|e| e.to_string())?;
    fs::write(&path, yaml).map_err(|e| format!("Failed to write config: {}", e))
}
