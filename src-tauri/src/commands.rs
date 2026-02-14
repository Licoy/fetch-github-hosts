use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;

use crate::config;
use crate::hosts;
use crate::models::{AppConfig, LogPayload, UpdateInfo};
use crate::services::{self, ClientState, ServerState};

#[tauri::command]
pub async fn start_client(
    app: AppHandle,
    url: String,
    interval: u32,
    state: State<'_, Mutex<ClientState>>,
) -> Result<(), String> {
    // Stop existing client if running
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        if let Some(tx) = s.stop_tx.take() {
            let _ = tx.send(());
        }
    }

    // Do initial fetch synchronously to catch privilege errors immediately
    services::client_fetch_hosts(&url).await?;

    // Emit success log for initial fetch
    let _ = app.emit("client-log", LogPayload {
        key: "client.fetchSuccess".to_string(),
        params: None,
        level: "success".to_string(),
    });

    let (tx, rx) = oneshot::channel();
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.stop_tx = Some(tx);
    }

    tokio::spawn(services::start_client_periodic_task(app, url, interval, rx));
    Ok(())
}

#[tauri::command]
pub async fn stop_client(state: State<'_, Mutex<ClientState>>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if let Some(tx) = s.stop_tx.take() {
        let _ = tx.send(());
    }
    Ok(())
}

#[tauri::command]
pub async fn start_server(
    app: AppHandle,
    port: u16,
    interval: u32,
    state: State<'_, Mutex<ServerState>>,
) -> Result<(), String> {
    // Stop existing server if running
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        if let Some(tx) = s.stop_tx.take() {
            let _ = tx.send(());
        }
    }

    let (tx, rx) = oneshot::channel();
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.stop_tx = Some(tx);
    }

    tokio::spawn(services::start_server_task(app, port, interval, rx));
    Ok(())
}

#[tauri::command]
pub async fn stop_server(state: State<'_, Mutex<ServerState>>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if let Some(tx) = s.stop_tx.take() {
        let _ = tx.send(());
    }
    Ok(())
}

#[tauri::command]
pub async fn clean_hosts() -> Result<(), String> {
    hosts::flush_clean_hosts()
}

#[tauri::command]
pub async fn check_permission() -> Result<bool, String> {
    hosts::check_hosts_permission()
}

#[tauri::command]
pub async fn flush_dns() -> Result<String, String> {
    hosts::flush_dns_cache()
}

#[tauri::command]
pub async fn load_config() -> Result<AppConfig, String> {
    Ok(config::load_config())
}

#[tauri::command]
pub async fn save_config(config_data: AppConfig) -> Result<(), String> {
    config::save_config(&config_data)
}

#[tauri::command]
pub fn get_version() -> String {
    crate::APP_VERSION.to_string()
}

#[tauri::command]
pub async fn check_update() -> Result<UpdateInfo, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.github.com/repos/Licoy/fetch-github-hosts/releases/latest")
        .header("User-Agent", format!("fetch-github-hosts/{}", env!("CARGO_PKG_VERSION")))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("网络请求错误: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("请求失败: HTTP {}", resp.status()));
    }

    let latest: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let tag = latest["tag_name"]
        .as_str()
        .unwrap_or("0")
        .trim_start_matches('v')
        .trim_start_matches('V');

    let has_update = crate::version_gt(tag, env!("CARGO_PKG_VERSION"));
    let html_url = latest["html_url"].as_str().unwrap_or("").to_string();

    Ok(UpdateInfo {
        has_update,
        version: tag.to_string(),
        url: html_url,
    })
}

/// Append a log entry to persistent log file
#[tauri::command]
pub async fn append_log(source: String, entry: String) -> Result<(), String> {
    config::append_log(&source, &entry)
}

/// Load persisted logs from file
#[tauri::command]
pub async fn load_logs(source: String) -> Result<Vec<String>, String> {
    config::load_logs(&source)
}

/// Clear persisted logs
#[tauri::command]
pub async fn clear_logs(source: String) -> Result<(), String> {
    config::clear_logs(&source)
}

/// Get the default server HTML template content
#[tauri::command]
pub fn get_default_template() -> String {
    services::get_default_template()
}

/// Export default template to a file in config directory and return file path
#[tauri::command]
pub fn export_default_template() -> Result<String, String> {
    let config_dir = config::config_dir();
    let template_path = config_dir.join("server_template.html");
    let content = services::get_default_template();
    std::fs::write(&template_path, &content)
        .map_err(|e| format!("Failed to write template file: {}", e))?;
    Ok(template_path.to_string_lossy().to_string())
}

/// Copy text to system clipboard
#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(&text).map_err(|e| e.to_string())
}
