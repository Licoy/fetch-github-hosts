use clap::Parser;
use crate::hosts;
use crate::services;

#[derive(Parser, Debug)]
#[command(
    name = "fetch-github-hosts",
    version = env!("CARGO_PKG_VERSION"),
    about = "GitHub Hosts synchronization tool / Github Hosts 同步工具",
    long_about = "A tool to help researchers and learners access Github faster by syncing DNS hosts.\n为解决研究及学习人员访问 Github 过慢或其他问题而提供的 Github Hosts 同步工具"
)]
pub struct CliArgs {
    /// Start mode: client or server (omit for GUI mode)
    /// 启动模式: client（客户端）或 server（服务端），不指定则启动 GUI
    #[arg(short, long)]
    pub mode: Option<String>,

    /// Fetch interval in minutes
    /// 获取 hosts 的间隔时间（分钟）
    #[arg(short, long, default_value = "60")]
    pub interval: u32,

    /// Server mode: listening port
    /// 服务端模式监听端口
    #[arg(short, long, default_value = "9898")]
    pub port: u16,

    /// Client mode: remote hosts URL
    /// 客户端模式远程 hosts 获取链接
    #[arg(short, long, default_value = "https://hosts.gitcdn.top/hosts.txt")]
    pub url: String,

    /// Interface language (zh-CN, en-US, ja-JP)
    /// 界面语言
    #[arg(short, long)]
    pub lang: Option<String>,
}

/// Run CLI mode (no GUI)
pub async fn run_cli(args: CliArgs) {
    let mode = args.mode.as_deref().unwrap_or("client");

    // Validate mode
    let mode = match mode {
        "client" | "server" => mode,
        other => {
            println!("⚠️  无效的启动模式: {}，已自动设置为 client", other);
            "client"
        }
    };

    // Validate interval
    let interval = if args.interval < 1 {
        println!("⚠️  获取间隔不可小于 1 分钟，已自动设置为 60 分钟");
        60
    } else {
        args.interval
    };

    println!("╔════════════════════════════════════════════════╗");
    println!("║        Fetch Github Hosts  V{}               ║", crate::APP_VERSION);
    println!("╚════════════════════════════════════════════════╝");
    println!();

    match mode {
        "server" => run_server_cli(args.port, interval).await,
        _ => run_client_cli(&args.url, interval).await,
    }
}

/// CLI Client mode: fetch hosts from URL, write to system, loop with interval
async fn run_client_cli(url: &str, interval_minutes: u32) {
    println!("🔄 客户端模式启动");
    println!("   远程地址: {}", url);
    println!("   更新间隔: {} 分钟", interval_minutes);
    println!("   请不要关闭此窗口以保持运行");
    println!();

    // Initial fetch
    cli_log("开始获取 GitHub Hosts...");
    match services::client_fetch_hosts(url).await {
        Ok(_) => cli_log("✅ 更新 Github-Hosts 成功！"),
        Err(e) => cli_log(&format!("❌ 更新 Github-Hosts 失败: {}", e)),
    }

    let interval = std::time::Duration::from_secs(interval_minutes as u64 * 60);
    let mut interval_timer = tokio::time::interval(interval);
    interval_timer.tick().await; // skip first tick

    // Handle Ctrl+C gracefully
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        let _ = shutdown_tx.send(true);
    });

    loop {
        tokio::select! {
            _ = interval_timer.tick() => {
                cli_log("开始获取 GitHub Hosts...");
                match services::client_fetch_hosts(url).await {
                    Ok(_) => cli_log("✅ 更新 Github-Hosts 成功！"),
                    Err(e) => cli_log(&format!("❌ 更新 Github-Hosts 失败: {}", e)),
                }
            }
            _ = shutdown_rx.changed() => {
                cli_log("🛑 收到停止信号，正在退出...");
                break;
            }
        }
    }

    // Cleanup sudoers on exit
    #[cfg(target_os = "macos")]
    hosts::cleanup_privileges();
}

/// CLI Server mode: resolve DNS, start HTTP, loop
async fn run_server_cli(port: u16, interval_minutes: u32) {
    println!("🌐 服务端模式启动");
    println!("   监听端口: {}", port);
    println!("   更新间隔: {} 分钟", interval_minutes);
    println!();

    // Initial DNS resolve
    cli_log("开始解析 GitHub DNS...");
    match services::server_fetch_hosts().await {
        Ok(_) => cli_log("✅ 解析 Github DNS 成功！"),
        Err(e) => cli_log(&format!("❌ 解析 Github DNS 失败: {}", e)),
    }

    // Start HTTP server
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let shutdown_tx_signal = shutdown_tx.clone();

    let http_handle = tokio::spawn(async move {
        start_cli_http_server(port, shutdown_rx).await;
    });

    // Handle Ctrl+C
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        let _ = shutdown_tx_signal.send(true);
    });

    cli_log(&format!("✅ HTTP 服务已启动: http://127.0.0.1:{}", port));
    cli_log(&format!("   hosts.txt → http://127.0.0.1:{}/hosts.txt", port));
    cli_log(&format!("   hosts.json → http://127.0.0.1:{}/hosts.json", port));

    let interval = std::time::Duration::from_secs(interval_minutes as u64 * 60);
    let mut interval_timer = tokio::time::interval(interval);
    interval_timer.tick().await; // skip first tick

    let mut shutdown_main = shutdown_tx.subscribe();

    loop {
        tokio::select! {
            _ = interval_timer.tick() => {
                cli_log("开始解析 GitHub DNS...");
                match services::server_fetch_hosts().await {
                    Ok(_) => cli_log("✅ 解析 Github DNS 成功！"),
                    Err(e) => cli_log(&format!("❌ 解析 Github DNS 失败: {}", e)),
                }
            }
            _ = shutdown_main.changed() => {
                cli_log("🛑 收到停止信号，正在退出...");
                let _ = shutdown_tx.send(true);
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(3),
                    http_handle,
                ).await;
                break;
            }
        }
    }
}

/// CLI HTTP server (no AppHandle dependency)
async fn start_cli_http_server(port: u16, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            cli_log(&format!("❌ HTTP 服务启动失败: {}", e));
            return;
        }
    };

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

                        let (status, content_type, body) = services::handle_http_request(path);

                        let response = format!(
                            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                            status, content_type, body.len(), body
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

/// Print log with timestamp to stdout
fn cli_log(msg: &str) {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("[{}] {}", now, msg);
}
