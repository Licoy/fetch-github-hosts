use std::fs;
use std::path::PathBuf;

use crate::models::AppConfig;

/// Get the config directory: ~/.fetch-github-hosts/
pub fn config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".fetch-github-hosts")
}

/// Get the config file path: ~/.fetch-github-hosts/config.yml
pub fn config_path() -> PathBuf {
    config_dir().join("config.yml")
}

/// Get the logs directory: ~/.fetch-github-hosts/logs/
pub fn logs_dir() -> PathBuf {
    config_dir().join("logs")
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

/// Append a log entry to a log file (source: "client" or "server")
pub fn append_log(source: &str, entry: &str) -> Result<(), String> {
    let dir = logs_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create log dir: {}", e))?;
    }
    let path = dir.join(format!("{}.log", source));
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;
    writeln!(file, "{}", entry).map_err(|e| format!("Failed to write log: {}", e))
}

/// Load logs from a log file, returning each line as a string
pub fn load_logs(source: &str) -> Result<Vec<String>, String> {
    let path = logs_dir().join(format!("{}.log", source));
    match fs::read_to_string(&path) {
        Ok(content) => Ok(content
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect()),
        Err(_) => Ok(vec![]),
    }
}

/// Clear a log file
pub fn clear_logs(source: &str) -> Result<(), String> {
    let path = logs_dir().join(format!("{}.log", source));
    if path.exists() {
        fs::write(&path, "").map_err(|e| format!("Failed to clear log: {}", e))?;
    }
    Ok(())
}
