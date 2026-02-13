use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

use crate::dns;
use crate::hosts;
use crate::models::LogPayload;

/// State for managing client ticker
pub struct ClientState {
    pub stop_tx: Option<oneshot::Sender<()>>,
}

/// State for managing server ticker
pub struct ServerState {
    pub stop_tx: Option<oneshot::Sender<()>>,
}

/// Start the client mode: periodically fetch hosts from remote URL
pub async fn start_client_task(
    app: AppHandle,
    url: String,
    interval_minutes: u32,
    stop_rx: oneshot::Receiver<()>,
) {
    let emit_log = |msg: String| {
        let _ = app.emit("client-log", LogPayload { message: msg });
    };

    // Initial fetch
    match client_fetch_hosts(&url).await {
        Ok(_) => emit_log("更新Github-Hosts成功！".to_string()),
        Err(e) => emit_log(format!("更新Github-Hosts失败: {}", e)),
    }

    let interval = std::time::Duration::from_secs(interval_minutes as u64 * 60);
    let mut interval_timer = tokio::time::interval(interval);
    interval_timer.tick().await; // skip first tick (already done above)

    tokio::select! {
        _ = async {
            loop {
                interval_timer.tick().await;
                match client_fetch_hosts(&url).await {
                    Ok(_) => emit_log("更新Github-Hosts成功！".to_string()),
                    Err(e) => emit_log(format!("更新Github-Hosts失败: {}", e)),
                }
            }
        } => {}
        _ = stop_rx => {
            emit_log("停止获取hosts".to_string());
        }
    }
}

/// Fetch hosts from remote URL and write to system hosts file
async fn client_fetch_hosts(url: &str) -> Result<(), String> {
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

    hosts::write_hosts(&result)
}

/// Start the server mode: resolve DNS and serve hosts via HTTP
pub async fn start_server_task(
    app: AppHandle,
    port: u16,
    interval_minutes: u32,
    stop_rx: oneshot::Receiver<()>,
) {
    let emit_log = |msg: String| {
        let _ = app.emit("server-log", LogPayload { message: msg });
    };

    // Initial fetch
    match server_fetch_hosts(&app).await {
        Ok(_) => emit_log("执行更新Github-Hosts成功！".to_string()),
        Err(e) => emit_log(format!("执行更新Github-Hosts失败：{}", e)),
    }

    // Start HTTP server in background
    let app_clone = app.clone();
    let http_handle = tokio::spawn(async move {
        start_http_server(port, app_clone).await;
    });

    let interval = std::time::Duration::from_secs(interval_minutes as u64 * 60);
    let mut interval_timer = tokio::time::interval(interval);
    interval_timer.tick().await;

    tokio::select! {
        _ = async {
            loop {
                interval_timer.tick().await;
                match server_fetch_hosts(&app).await {
                    Ok(_) => emit_log("执行更新Github-Hosts成功！".to_string()),
                    Err(e) => emit_log(format!("执行更新Github-Hosts失败：{}", e)),
                }
            }
        } => {}
        _ = stop_rx => {
            emit_log("已停止更新hosts服务".to_string());
            http_handle.abort();
        }
    }
}

/// Server-side: resolve DNS for GitHub domains, save to files
async fn server_fetch_hosts(_app: &AppHandle) -> Result<(), String> {
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

/// Simple HTTP server for serving hosts files
async fn start_http_server(port: u16, app: AppHandle) {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            let _ = app.emit(
                "server-log",
                LogPayload {
                    message: format!("服务启动失败：{}", e),
                },
            );
            return;
        }
    };

    // Set non-blocking so we can be aborted
    listener
        .set_nonblocking(true)
        .expect("cannot set non-blocking");

    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let request = String::from_utf8_lossy(&buf);

                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/");

                let exe_dir = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                    .unwrap_or_else(|| std::path::PathBuf::from("."));

                let (status, content_type, body) = match path {
                    "/hosts.txt" => {
                        let content = std::fs::read_to_string(exe_dir.join("hosts.txt"))
                            .unwrap_or_else(|_| "# no hosts yet".to_string());
                        ("200 OK", "text/plain", content)
                    }
                    "/hosts.json" => {
                        let content = std::fs::read_to_string(exe_dir.join("hosts.json"))
                            .unwrap_or_else(|_| "[]".to_string());
                        ("200 OK", "application/json", content)
                    }
                    _ => {
                        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        let content = format!(r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>Fetch Github Hosts</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
:root{{--bg:#0f1117;--card:#1a1d2e;--border:#2d3148;--text:#e2e8f0;--muted:rgba(255,255,255,0.5);--primary:#009966;--primary-glow:rgba(0,153,102,0.15)}}
.light{{--bg:#f5f7fa;--card:#ffffff;--border:#e2e8f0;--text:#1a202c;--muted:rgba(0,0,0,0.5);--primary-glow:rgba(0,153,102,0.1)}}
body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:var(--bg);color:var(--text);min-height:100vh;display:flex;align-items:center;justify-content:center;padding:2rem;transition:background .3s,color .3s}}
.container{{max-width:640px;width:100%}}
.toolbar{{display:flex;justify-content:flex-end;gap:0.5rem;margin-bottom:1.5rem}}
.toolbar button,.toolbar select{{background:var(--card);border:1px solid var(--border);color:var(--text);padding:0.35rem 0.7rem;border-radius:6px;font-size:0.8rem;cursor:pointer;transition:all .2s}}
.toolbar button:hover,.toolbar select:hover{{border-color:var(--primary)}}
.header{{text-align:center;margin-bottom:2.5rem}}
.logo-wrap{{display:inline-flex;align-items:center;justify-content:center;width:72px;height:72px;border-radius:18px;background:linear-gradient(135deg,var(--primary-glow),rgba(0,153,102,0.05));margin-bottom:1rem}}
.logo-wrap svg{{width:40px;height:40px;color:var(--primary)}}
h1{{font-size:1.5rem;font-weight:700;margin-bottom:0.5rem}}
.subtitle{{color:var(--muted);font-size:0.875rem;line-height:1.6;max-width:460px;margin:0 auto}}
.card{{background:var(--card);border:1px solid var(--border);border-radius:12px;padding:1.5rem;margin-bottom:1rem;transition:background .3s,border-color .3s}}
.card-title{{font-size:0.75rem;text-transform:uppercase;letter-spacing:0.05em;color:var(--muted);margin-bottom:1rem;font-weight:600}}
.link-row{{display:flex;align-items:center;justify-content:space-between;padding:0.75rem 0;border-bottom:1px solid var(--border);gap:0.75rem;flex-wrap:wrap}}
.link-row:last-child{{border-bottom:none}}
.link-info{{display:flex;align-items:center;gap:0.75rem;flex:1;min-width:0}}
.link-dot{{width:8px;height:8px;border-radius:50%;background:var(--primary);flex-shrink:0}}
.link-name{{font-weight:500;font-size:0.9rem}}
.link-desc{{font-size:0.75rem;color:var(--muted);margin-top:2px}}
a.link-btn{{display:inline-flex;align-items:center;gap:0.4rem;padding:0.4rem 0.9rem;background:var(--primary);color:#fff;border-radius:6px;font-size:0.8rem;text-decoration:none;font-weight:500;transition:filter 0.2s;white-space:nowrap;flex-shrink:0}}
a.link-btn:hover{{filter:brightness(1.15)}}
.time-badge{{display:inline-flex;align-items:center;gap:0.5rem;background:var(--primary-glow);border:1px solid rgba(0,153,102,0.2);border-radius:8px;padding:0.5rem 1rem;font-size:0.8rem;color:var(--primary);margin-top:1rem}}
.footer{{text-align:center;margin-top:2rem;font-size:0.75rem;color:var(--muted)}}
.footer a{{color:var(--primary);text-decoration:none}}
</style>
</head>
<body>
<div class="container">
  <div class="toolbar">
    <select id="lang" onchange="switchLang(this.value)">
      <option value="zh">简体中文</option>
      <option value="en">English</option>
      <option value="ja">日本語</option>
    </select>
    <button onclick="toggleTheme()" id="themeBtn">☀️</button>
  </div>
  <div class="header">
    <div class="logo-wrap">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"/>
      </svg>
    </div>
    <h1>Fetch Github Hosts</h1>
    <p class="subtitle" data-i18n="subtitle"></p>
    <div class="time-badge">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
      <span data-i18n="lastUpdate"></span> {now}
    </div>
  </div>
  <div class="card">
    <div class="card-title" data-i18n="resources"></div>
    <div class="link-row">
      <div class="link-info">
        <div class="link-dot"></div>
        <div>
          <div class="link-name">hosts.txt</div>
          <div class="link-desc" data-i18n="txtDesc"></div>
        </div>
      </div>
      <a href="/hosts.txt" target="_blank" class="link-btn">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        <span data-i18n="view"></span>
      </a>
    </div>
    <div class="link-row">
      <div class="link-info">
        <div class="link-dot" style="background:#3b82f6"></div>
        <div>
          <div class="link-name">hosts.json</div>
          <div class="link-desc" data-i18n="jsonDesc"></div>
        </div>
      </div>
      <a href="/hosts.json" target="_blank" class="link-btn" style="background:#3b82f6">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        <span data-i18n="view"></span>
      </a>
    </div>
  </div>
  <div class="footer">
    <div style="margin-bottom:0.5rem">
      <a href="https://github.com/Licoy/fetch-github-hosts/releases" target="_blank" style="display:inline-flex;align-items:center;gap:4px;padding:4px 12px;border-radius:6px;background:var(--primary);color:#fff;font-size:0.8rem;text-decoration:none">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        <span data-i18n="downloadClient"></span>
      </a>
    </div>
    Powered by <a href="https://github.com/Licoy/fetch-github-hosts" target="_blank">Fetch Github Hosts</a> V4.0
  </div>
</div>
<script>
const i18n={{
  zh:{{subtitle:"为解决研究及学习人员访问 GitHub 过慢或其他问题而提供的 Github Hosts 同步工具",lastUpdate:"最近更新：",resources:"可用资源",txtDesc:"纯文本格式，可直接追加到系统 hosts 文件",jsonDesc:"JSON 格式，适合程序化调用",view:"查看",downloadClient:"下载桌面客户端"}},
  en:{{subtitle:"A Github Hosts synchronization tool to help researchers and learners access GitHub faster",lastUpdate:"Last update: ",resources:"AVAILABLE RESOURCES",txtDesc:"Plain text format, can be appended to system hosts file",jsonDesc:"JSON format, suitable for programmatic use",view:"View",downloadClient:"Download Desktop Client"}},
  ja:{{subtitle:"研究者や学習者がGitHubへのアクセスを高速化するためのGithub Hosts同期ツール",lastUpdate:"最終更新：",resources:"利用可能なリソース",txtDesc:"プレーンテキスト形式、システムhostsファイルに追記可能",jsonDesc:"JSON形式、プログラムでの利用に適しています",view:"表示",downloadClient:"デスクトップクライアントをダウンロード"}}
}};
function getStoredOrDefault(key,detectFn){{return localStorage.getItem('fgh_'+key)||detectFn()}}
function detectLang(){{const l=navigator.language||'zh-CN';if(l.startsWith('ja'))return'ja';if(l.startsWith('en'))return'en';return'zh'}}
function detectTheme(){{return window.matchMedia&&window.matchMedia('(prefers-color-scheme:light)').matches?'light':'dark'}}
let isDark=getStoredOrDefault('theme',detectTheme)==='dark';
document.body.classList.toggle('light',!isDark);
document.getElementById('themeBtn').textContent=isDark?'☀️':'🌙';
function toggleTheme(){{isDark=!isDark;document.body.classList.toggle('light',!isDark);document.getElementById('themeBtn').textContent=isDark?'☀️':'🌙';localStorage.setItem('fgh_theme',isDark?'dark':'light')}}
function switchLang(l){{const t=i18n[l]||i18n.zh;document.querySelectorAll('[data-i18n]').forEach(el=>{{const k=el.dataset.i18n;if(t[k])el.textContent=t[k]}});document.getElementById('lang').value=l;localStorage.setItem('fgh_lang',l)}}
(function(){{const lang=getStoredOrDefault('lang',detectLang);switchLang(lang)}})();
</script>
</body>
</html>"##, now = now);
                        ("200 OK", "text/html; charset=utf-8", content)
                    }
                };

                let response = format!(
                    "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                    status,
                    content_type,
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Err(_) => break,
        }
    }
}
