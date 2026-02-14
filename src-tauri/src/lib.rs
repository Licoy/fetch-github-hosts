pub mod cli;
#[cfg(feature = "gui")]
mod commands;
pub(crate) mod config;
pub(crate) mod dns;
pub(crate) mod hosts;
pub(crate) mod models;
pub(crate) mod services;

/// Application version (read from Cargo.toml at compile time)
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compare two semver-like version strings: returns true if remote > local
pub fn version_gt(remote: &str, local: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> {
        s.split('.')
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };
    let r = parse(remote);
    let l = parse(local);
    let max_len = r.len().max(l.len());
    for i in 0..max_len {
        let rv = r.get(i).copied().unwrap_or(0);
        let lv = l.get(i).copied().unwrap_or(0);
        if rv > lv {
            return true;
        }
        if rv < lv {
            return false;
        }
    }
    false
}

#[cfg(feature = "gui")]
use std::sync::Mutex;
#[cfg(feature = "gui")]
use services::{ClientState, ServerState};
#[cfg(feature = "gui")]
use tauri::{
    Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};

/// Detect system language and return supported locale code
pub fn detect_system_lang() -> &'static str {
    let lang = sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string());
    if lang.starts_with("zh") {
        "zh-CN"
    } else if lang.starts_with("ja") {
        "ja-JP"
    } else {
        "en-US"
    }
}

/// Known hosts origins (must match frontend)
#[cfg(feature = "gui")]
fn get_hosts_url(select_origin: &str, custom_url: &str, method: &str) -> String {
    if method == "custom" && !custom_url.is_empty() {
        return custom_url.to_string();
    }
    match select_origin {
        "Github520" => "https://raw.hellogithub.com/hosts".to_string(),
        _ => "https://hosts.gitcdn.top/hosts.txt".to_string(), // FetchGithubHosts default
    }
}

/// Get tray menu text by key and locale
#[cfg(feature = "gui")]
fn tray_text(key: &str, lang: &str) -> &'static str {
    match (key, lang) {
        ("show", "zh-CN") => "显示主窗口",
        ("show", "en-US") => "Show Window",
        ("show", "ja-JP") => "ウィンドウを表示",

        ("start_client", "zh-CN") => "启动客户端",
        ("start_client", "en-US") => "Start Client",
        ("start_client", "ja-JP") => "クライアント起動",

        ("stop_client", "zh-CN") => "停止客户端",
        ("stop_client", "en-US") => "Stop Client",
        ("stop_client", "ja-JP") => "クライアント停止",

        ("start_server", "zh-CN") => "启动服务端",
        ("start_server", "en-US") => "Start Server",
        ("start_server", "ja-JP") => "サーバー起動",

        ("stop_server", "zh-CN") => "停止服务端",
        ("stop_server", "en-US") => "Stop Server",
        ("stop_server", "ja-JP") => "サーバー停止",

        ("flush_dns", "zh-CN") => "刷新 DNS",
        ("flush_dns", "en-US") => "Flush DNS",
        ("flush_dns", "ja-JP") => "DNS をフラッシュ",

        ("clean_hosts", "zh-CN") => "清空 Hosts",
        ("clean_hosts", "en-US") => "Clean Hosts",
        ("clean_hosts", "ja-JP") => "Hosts をクリア",

        ("quit", "zh-CN") => "退出",
        ("quit", "en-US") => "Quit",
        ("quit", "ja-JP") => "終了",

        _ => "Unknown",
    }
}

#[cfg(feature = "gui")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(ClientState { stop_tx: None }))
        .manage(Mutex::new(ServerState { stop_tx: None }))
        .invoke_handler(tauri::generate_handler![
            commands::start_client,
            commands::stop_client,
            commands::start_server,
            commands::stop_server,
            commands::clean_hosts,
            commands::check_permission,
            commands::flush_dns,
            commands::load_config,
            commands::save_config,
            commands::get_version,
            commands::check_update,
        ])
        .setup(|app| {
            // Logger plugin (debug only)
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let lang = detect_system_lang();

            // Build tray menu items
            let show_item = MenuItemBuilder::with_id("show", tray_text("show", lang))
                .build(app)?;

            let start_client_item = MenuItemBuilder::with_id("start_client", tray_text("start_client", lang))
                .build(app)?;
            let stop_client_item = MenuItemBuilder::with_id("stop_client", tray_text("stop_client", lang))
                .build(app)?;

            let start_server_item = MenuItemBuilder::with_id("start_server", tray_text("start_server", lang))
                .build(app)?;
            let stop_server_item = MenuItemBuilder::with_id("stop_server", tray_text("stop_server", lang))
                .build(app)?;

            let flush_dns_item = MenuItemBuilder::with_id("flush_dns", tray_text("flush_dns", lang))
                .build(app)?;
            let clean_hosts_item = MenuItemBuilder::with_id("clean_hosts", tray_text("clean_hosts", lang))
                .build(app)?;

            let quit_item = MenuItemBuilder::with_id("quit", tray_text("quit", lang))
                .build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&start_client_item)
                .item(&stop_client_item)
                .separator()
                .item(&start_server_item)
                .item(&stop_server_item)
                .separator()
                .item(&flush_dns_item)
                .item(&clean_hosts_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("Fetch Github Hosts")
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "start_client" => {
                            let app = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let cfg = config::load_config();
                                let url = get_hosts_url(
                                    &cfg.client.select_origin,
                                    &cfg.client.custom_url,
                                    &cfg.client.method,
                                );
                                let state = app.state::<Mutex<ClientState>>();
                                // Stop existing client if running
                                {
                                    if let Ok(mut s) = state.lock() {
                                        if let Some(tx) = s.stop_tx.take() {
                                            let _ = tx.send(());
                                        }
                                    }
                                }
                                let (tx, rx) = tokio::sync::oneshot::channel();
                                {
                                    if let Ok(mut s) = state.lock() {
                                        s.stop_tx = Some(tx);
                                    }
                                }
                                let _ = services::start_client_task(app, url, cfg.client.interval, rx).await;
                            });
                        }
                        "stop_client" => {
                            let state = app.state::<Mutex<ClientState>>();
                            let mut guard = state.lock().expect("lock client state");
                            if let Some(tx) = guard.stop_tx.take() {
                                let _ = tx.send(());
                            }
                            drop(guard);
                        }
                        "start_server" => {
                            let app = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let cfg = config::load_config();
                                let state = app.state::<Mutex<ServerState>>();
                                {
                                    if let Ok(mut s) = state.lock() {
                                        if let Some(tx) = s.stop_tx.take() {
                                            let _ = tx.send(());
                                        }
                                    }
                                }
                                let (tx, rx) = tokio::sync::oneshot::channel();
                                {
                                    if let Ok(mut s) = state.lock() {
                                        s.stop_tx = Some(tx);
                                    }
                                }
                                let _ = services::start_server_task(app, cfg.server.port, cfg.server.interval, rx).await;
                            });
                        }
                        "stop_server" => {
                            let state = app.state::<Mutex<ServerState>>();
                            let mut guard = state.lock().expect("lock server state");
                            if let Some(tx) = guard.stop_tx.take() {
                                let _ = tx.send(());
                            }
                            drop(guard);
                        }
                        "flush_dns" => {
                            tauri::async_runtime::spawn(async {
                                let _ = hosts::flush_dns_cache();
                            });
                        }
                        "clean_hosts" => {
                            tauri::async_runtime::spawn(async {
                                let _ = hosts::flush_clean_hosts();
                            });
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // Clean up temporary sudoers entry on exit
    #[cfg(target_os = "macos")]
    hosts::cleanup_privileges();
}
