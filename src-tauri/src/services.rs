#[cfg(feature = "gui")]
use tauri::{AppHandle, Emitter};
#[cfg(feature = "gui")]
use tokio::sync::oneshot;

use crate::dns;
use crate::hosts;
use crate::config;
#[cfg(feature = "gui")]
use crate::models::LogPayload;

/// State for managing client ticker
#[cfg(feature = "gui")]
pub struct ClientState {
    pub stop_tx: Option<oneshot::Sender<()>>,
}

/// State for managing server ticker
#[cfg(feature = "gui")]
pub struct ServerState {
    pub stop_tx: Option<oneshot::Sender<()>>,
}

/// Helper to create a LogPayload
#[cfg(feature = "gui")]
fn log_payload(key: &str, params: Option<serde_json::Value>, level: &str) -> LogPayload {
    LogPayload {
        key: key.to_string(),
        params,
        level: level.to_string(),
    }
}

/// Start the client periodic task (initial fetch already done in command handler)
#[cfg(feature = "gui")]
pub async fn start_client_periodic_task(
    app: AppHandle,
    url: String,
    interval_minutes: u32,
    stop_rx: oneshot::Receiver<()>,
) {
    let emit_log = |key: &str, params: Option<serde_json::Value>, level: &str| {
        let _ = app.emit("client-log", log_payload(key, params, level));
    };

    let interval = std::time::Duration::from_secs(interval_minutes as u64 * 60);
    let mut interval_timer = tokio::time::interval(interval);
    interval_timer.tick().await; // skip first tick (initial fetch done by command handler)

    tokio::select! {
        _ = async {
            loop {
                interval_timer.tick().await;
                match client_fetch_hosts(&url).await {
                    Ok(_) => emit_log("client.fetchSuccess", None, "success"),
                    Err(e) => emit_log("client.fetchFail", Some(serde_json::json!({"error": e})), "error"),
                }
            }
        } => {}
        _ = stop_rx => {
            emit_log("client.fetchStop", None, "info");
        }
    }
}

/// Fetch hosts from remote URL and write to system hosts file
pub async fn client_fetch_hosts(url: &str) -> Result<(), String> {
    let clean_hosts = hosts::get_clean_hosts()?;

    let resp = reqwest::get(url)
        .await
        .map_err(|e| format!("获取最新的hosts失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("获取最新的hosts失败: HTTP {}", resp.status()));
    }

    let fetch_hosts = resp
        .text()
        .await
        .map_err(|e| format!("读取最新的hosts失败: {}", e))?;

    let newline = hosts::newline_char();
    let mut result = clean_hosts;

    for line in fetch_hosts.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        result.push_str(line);
        result.push_str(newline);
    }

    hosts::write_hosts(&result)?;

    // Auto flush DNS cache after writing hosts
    let _ = hosts::flush_dns_cache();

    Ok(())
}

/// Start the server mode: resolve DNS and serve hosts via HTTP
#[cfg(feature = "gui")]
pub async fn start_server_task(
    app: AppHandle,
    port: u16,
    interval_minutes: u32,
    stop_rx: oneshot::Receiver<()>,
) {
    let emit_log = |key: &str, params: Option<serde_json::Value>, level: &str| {
        let _ = app.emit("server-log", log_payload(key, params, level));
    };

    // Initial fetch
    match server_fetch_hosts().await {
        Ok(_) => emit_log("server.fetchSuccess", None, "success"),
        Err(e) => emit_log("server.fetchFail", Some(serde_json::json!({"error": e})), "error"),
    }

    // Shared shutdown signal via tokio::sync::watch
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // Start HTTP server in background
    let app_clone = app.clone();
    let http_handle = tokio::spawn(async move {
        start_http_server(port, app_clone, shutdown_rx).await;
    });

    let interval = std::time::Duration::from_secs(interval_minutes as u64 * 60);
    let mut interval_timer = tokio::time::interval(interval);
    interval_timer.tick().await;

    tokio::select! {
        _ = async {
            loop {
                interval_timer.tick().await;
                match server_fetch_hosts().await {
                    Ok(_) => emit_log("server.fetchSuccess", None, "success"),
                    Err(e) => emit_log("server.fetchFail", Some(serde_json::json!({"error": e})), "error"),
                }
            }
        } => {}
        _ = stop_rx => {
            emit_log("server.stopSuccess", None, "info");
            // Signal shutdown to HTTP server gracefully
            let _ = shutdown_tx.send(true);
            // Wait for HTTP server to finish (with timeout)
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(3),
                http_handle,
            ).await;
        }
    }
}

/// Server-side: resolve DNS for GitHub domains, save to files
pub async fn server_fetch_hosts() -> Result<(), String> {
    let domains = dns::get_github_domains();
    let hosts = dns::fetch_hosts(&domains);
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let hosts_json = serde_json::to_string(&hosts).map_err(|e| e.to_string())?;
    let hosts_text = dns::format_hosts_text(&hosts, &now);

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    std::fs::write(exe_dir.join("hosts.json"), &hosts_json)
        .map_err(|e| format!("写入hosts.json失败: {}", e))?;
    std::fs::write(exe_dir.join("hosts.txt"), &hosts_text)
        .map_err(|e| format!("写入hosts.txt失败: {}", e))?;

    Ok(())
}

/// Simple HTTP server for serving hosts files (async with graceful shutdown)
#[cfg(feature = "gui")]
async fn start_http_server(port: u16, app: AppHandle, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            let _ = app.emit(
                "server-log",
                log_payload("server.startFail", Some(serde_json::json!({"error": e.to_string()})), "error"),
            );
            return;
        }
    };

    let _ = app.emit(
        "server-log",
        log_payload("server.httpStarted", Some(serde_json::json!({"port": port})), "success"),
    );

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((mut stream, _)) => {
                        let mut buf = [0u8; 4096];
                        let _ = stream.read(&mut buf).await;
                        let request = String::from_utf8_lossy(&buf);

                        let path = request
                            .lines()
                            .next()
                            .and_then(|line| line.split_whitespace().nth(1))
                            .unwrap_or("/");

                        let (status, content_type, body) = handle_http_request(path);

                        let response = format!(
                            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                            status,
                            content_type,
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                    }
                    Err(_) => break,
                }
            }
            _ = shutdown_rx.changed() => {
                break;
            }
        }
    }
}

/// Handle HTTP request and return (status, content_type, body)
pub fn handle_http_request(path: &str) -> (String, String, String) {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    match path {
        "/hosts.txt" => {
            let content = std::fs::read_to_string(exe_dir.join("hosts.txt"))
                .unwrap_or_else(|_| "# no hosts yet".to_string());
            ("200 OK".to_string(), "text/plain".to_string(), content)
        }
        "/hosts.json" => {
            let content = std::fs::read_to_string(exe_dir.join("hosts.json"))
                .unwrap_or_else(|_| "[]".to_string());
            ("200 OK".to_string(), "application/json".to_string(), content)
        }
        _ => {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let content = generate_server_html(&now);
            ("200 OK".to_string(), "text/html; charset=utf-8".to_string(), content)
        }
    }
}

/// Generate the server HTML page content, supporting custom template
pub fn generate_server_html(now: &str) -> String {
    let cfg = config::load_config();
    let template_path = &cfg.server.template_path;

    // Try loading custom template if configured
    if !template_path.is_empty() {
        if let Ok(template) = std::fs::read_to_string(template_path) {
            return template
                .replace("{{FGH_VERSION}}", crate::APP_VERSION)
                .replace("{{FGH_UPDATE_TIME}}", now);
        }
    }

    // Use built-in default template
    let template = include_str!("../assets/server_template.html");
    template
        .replace("{{FGH_VERSION}}", crate::APP_VERSION)
        .replace("{{FGH_UPDATE_TIME}}", now)
}

/// Get the default template content (for user to export/customize)
pub fn get_default_template() -> String {
    include_str!("../assets/server_template.html").to_string()
}
