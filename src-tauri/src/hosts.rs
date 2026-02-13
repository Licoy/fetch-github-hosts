use std::fs;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

/// Track whether we've already granted elevated privileges this session
static PRIVILEGES_GRANTED: AtomicBool = AtomicBool::new(false);

/// Sudoers temp file path
#[cfg(target_os = "macos")]
const SUDOERS_TEMP: &str = "/etc/sudoers.d/fgh-temp";

/// Ensure we have passwordless sudo for FGH operations (macOS only)
/// Returns Ok(true) if privileges are now available
#[cfg(target_os = "macos")]
pub fn ensure_elevated() -> Result<bool, String> {
    if PRIVILEGES_GRANTED.load(Ordering::Relaxed) {
        return Ok(true);
    }

    // Get current username
    let username = std::env::var("USER").unwrap_or_else(|_| "root".to_string());

    // Use osascript to create a sudoers.d entry for passwordless operations
    let sudoers_content = format!(
        "{} ALL=(ALL) NOPASSWD: /usr/bin/tee /etc/hosts, /bin/cp * /etc/hosts, /usr/bin/killall -HUP mDNSResponder, /usr/bin/dscacheutil -flushcache",
        username
    );

    let script = format!(
        "do shell script \"echo '{}' > {} && chmod 440 {}\" with administrator privileges",
        sudoers_content, SUDOERS_TEMP, SUDOERS_TEMP
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to request privileges: {}", e))?;

    if output.status.success() {
        PRIVILEGES_GRANTED.store(true, Ordering::Relaxed);
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("User canceled") || stderr.contains("cancelled") {
            Err("USER_CANCELLED".to_string())
        } else {
            Err(format!("Failed to grant privileges: {}", stderr))
        }
    }
}

/// Clean up the temporary sudoers file on app exit
#[cfg(target_os = "macos")]
pub fn cleanup_privileges() {
    if PRIVILEGES_GRANTED.load(Ordering::Relaxed) {
        let _ = std::process::Command::new("sudo")
            .arg("rm")
            .arg("-f")
            .arg(SUDOERS_TEMP)
            .output();
        PRIVILEGES_GRANTED.store(false, Ordering::Relaxed);
    }
}

/// Get the system hosts file path
pub fn hosts_path() -> String {
    #[cfg(target_os = "windows")]
    {
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
        format!("{}\\System32\\drivers\\etc\\hosts", system_root)
    }
    #[cfg(not(target_os = "windows"))]
    {
        "/etc/hosts".to_string()
    }
}

/// Read the hosts file and remove any existing fetch-github-hosts entries
pub fn get_clean_hosts() -> Result<String, String> {
    let path = hosts_path();
    let content = fs::read_to_string(&path).map_err(|e| format!("读取hosts文件失败: {}", e))?;

    let mut result = String::new();
    let mut in_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "# fetch-github-hosts begin" {
            in_block = true;
            continue;
        }
        if in_block {
            if trimmed.starts_with("# fetch-github-hosts end") {
                in_block = false;
                continue;
            }
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }

    Ok(result)
}

/// Write content to the system hosts file (with elevation if needed)
pub fn write_hosts(content: &str) -> Result<(), String> {
    let path = hosts_path();
    // Try direct write first
    if fs::write(&path, content).is_ok() {
        return Ok(());
    }
    // Direct write failed, try elevated write
    write_hosts_elevated(content, &path)
}

/// Write hosts file with elevated privileges using OS-specific mechanisms
fn write_hosts_elevated(content: &str, hosts_path: &str) -> Result<(), String> {
    // Write content to a temp file first
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("fgh_hosts_temp.txt");
    let temp_path_str = temp_path.to_string_lossy().to_string();

    let mut temp_file = fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    temp_file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    drop(temp_file);

    #[cfg(target_os = "macos")]
    {
        // Ensure we have passwordless sudo for this session
        ensure_elevated()?;

        let output = std::process::Command::new("sudo")
            .arg("cp")
            .arg(&temp_path_str)
            .arg(hosts_path)
            .output()
            .map_err(|e| format!("Failed to copy hosts: {}", e))?;

        let _ = fs::remove_file(&temp_path);

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to write hosts (elevated): {}", stderr))
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = std::process::Command::new("pkexec")
            .arg("cp")
            .arg(&temp_path_str)
            .arg(hosts_path)
            .output()
            .map_err(|e| format!("Failed to run pkexec: {}", e))?;

        let _ = fs::remove_file(&temp_path);

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to write hosts (elevated): {}", stderr))
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, try PowerShell with elevated copy
        let output = std::process::Command::new("powershell")
            .arg("-Command")
            .arg(format!(
                "Start-Process powershell -Verb RunAs -ArgumentList '-Command','Copy-Item -Path \"{}\" -Destination \"{}\" -Force' -Wait",
                temp_path_str, hosts_path
            ))
            .output()
            .map_err(|e| format!("Failed to run elevated command: {}", e))?;

        let _ = fs::remove_file(&temp_path);

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to write hosts (elevated): {}", stderr))
        }
    }
}

/// Clean fetch-github-hosts entries from hosts file and write back
pub fn flush_clean_hosts() -> Result<(), String> {
    let clean = get_clean_hosts()?;
    write_hosts(&clean)
}

/// Check if we have read/write permission to the hosts file
pub fn check_hosts_permission() -> Result<bool, String> {
    let path = hosts_path();
    match fs::OpenOptions::new().read(true).write(true).open(&path) {
        Ok(_) => Ok(true),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Permission denied") || msg.contains("Access is denied") {
                Ok(false)
            } else {
                Err(format!("检查权限失败: {}", e))
            }
        }
    }
}

/// Get the newline character for the current platform
pub fn newline_char() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "\r\n"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "\n"
    }
}

/// Flush the system DNS cache
pub fn flush_dns_cache() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        // Ensure we have passwordless sudo for this session
        ensure_elevated()?;

        // Now use sudo (passwordless) to flush DNS
        let _ = std::process::Command::new("sudo")
            .arg("dscacheutil")
            .arg("-flushcache")
            .output();

        let output = std::process::Command::new("sudo")
            .arg("killall")
            .arg("-HUP")
            .arg("mDNSResponder")
            .output()
            .map_err(|e| format!("Failed to flush DNS: {}", e))?;

        if output.status.success() {
            Ok("DNS cache flushed successfully".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to flush DNS: {}", stderr))
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try systemd-resolved first, then nscd
        let output = std::process::Command::new("systemd-resolve")
            .arg("--flush-caches")
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                return Ok("DNS cache flushed (systemd-resolved)".to_string());
            }
        }

        let output = std::process::Command::new("resolvectl")
            .arg("flush-caches")
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                return Ok("DNS cache flushed (resolvectl)".to_string());
            }
        }

        // Try nscd
        let _ = std::process::Command::new("pkexec")
            .arg("service")
            .arg("nscd")
            .arg("restart")
            .output();

        Ok("DNS cache flush attempted".to_string())
    }

    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("ipconfig")
            .arg("/flushdns")
            .output()
            .map_err(|e| format!("Failed to flush DNS: {}", e))?;

        if output.status.success() {
            Ok("DNS cache flushed successfully".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to flush DNS: {}", stderr))
        }
    }
}
