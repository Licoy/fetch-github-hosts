use std::sync::Mutex;
use tauri::{AppHandle, State};
use tokio::sync::oneshot;

use crate::config;
use crate::hosts;
use crate::models::{AppConfig, UpdateInfo};
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

    let (tx, rx) = oneshot::channel();
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.stop_tx = Some(tx);
    }

    tokio::spawn(services::start_client_task(app, url, interval, rx));
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
pub async fn check_update() -> Result<UpdateInfo, String> {
    let resp = reqwest::get("https://api.github.com/repos/Licoy/fetch-github-hosts/releases")
        .await
        .map_err(|e| format!("网络请求错误: {}", e))?;

    if !resp.status().is_success() {
        return Err("请求失败".to_string());
    }

    let releases: Vec<serde_json::Value> = resp
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if releases.is_empty() {
        return Err("检查更新失败: 无发布版本".to_string());
    }

    let latest = &releases[0];
    let tag = latest["tag_name"]
        .as_str()
        .unwrap_or("0")
        .trim_start_matches('v')
        .trim_start_matches('V');

    let remote_version: f64 = tag.parse().unwrap_or(0.0);
    let current_version: f64 = 4.0;

    let html_url = latest["html_url"].as_str().unwrap_or("").to_string();

    Ok(UpdateInfo {
        has_update: remote_version > current_version,
        version: tag.to_string(),
        url: html_url,
    })
}
