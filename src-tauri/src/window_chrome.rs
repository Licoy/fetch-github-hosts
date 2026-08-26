#[cfg(any(windows, target_os = "linux"))]
use tauri::Manager;

/// Fallback surfaces used before the frontend token is known.
#[cfg(any(windows, test))]
const LIGHT_BG: &str = "#ffffff";
#[cfg(any(windows, test))]
const DARK_BG: &str = "#1f2437";

#[cfg(any(windows, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

/// Parse `#rgb`, `#rrggbb`, `#rrggbbaa`, `rgb()`, or `rgba()` into 8-bit sRGB.
#[cfg(any(windows, test))]
fn parse_css_color(input: &str) -> Option<Rgb> {
    let s = input.trim();
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex_color(hex);
    }
    let lower = s.to_ascii_lowercase();
    let inner = lower
        .strip_prefix("rgba(")
        .or_else(|| lower.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    let mut parts = inner.split(',');
    let r = parse_rgb_component(parts.next()?)?;
    let g = parse_rgb_component(parts.next()?)?;
    let b = parse_rgb_component(parts.next()?)?;
    Some(Rgb { r, g, b })
}

#[cfg(any(windows, test))]
fn parse_hex_color(hex: &str) -> Option<Rgb> {
    match hex.len() {
        3 => Some(Rgb {
            r: u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?,
            g: u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?,
            b: u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?,
        }),
        6 | 8 => Some(Rgb {
            r: u8::from_str_radix(&hex[0..2], 16).ok()?,
            g: u8::from_str_radix(&hex[2..4], 16).ok()?,
            b: u8::from_str_radix(&hex[4..6], 16).ok()?,
        }),
        _ => None,
    }
}

#[cfg(any(windows, test))]
fn parse_rgb_component(s: &str) -> Option<u8> {
    let t = s.trim();
    if let Some(pct) = t.strip_suffix('%') {
        let value: f32 = pct.trim().parse().ok()?;
        return Some((value.clamp(0.0, 100.0) * 2.55).round() as u8);
    }
    t.parse().ok()
}

/// Win32 COLORREF is `0x00BBGGRR`, not `#RRGGBB`.
#[cfg(any(windows, test))]
fn to_colorref(rgb: Rgb) -> u32 {
    u32::from(rgb.r) | (u32::from(rgb.g) << 8) | (u32::from(rgb.b) << 16)
}

#[cfg(any(windows, test))]
fn resolve_surface(bg: &str, dark: bool) -> Rgb {
    parse_css_color(bg).unwrap_or_else(|| {
        parse_css_color(if dark { DARK_BG } else { LIGHT_BG })
            .expect("built-in window surface colors are valid")
    })
}

/// Windows / Linux keep a native caption unless decorations are off.
/// `titleBarStyle: Overlay` is macOS-only, so the custom title bar would
/// otherwise sit *inside* the system chrome. Shadow stays on for Win11
/// rounded corners. The 1px DWM caption leftover is painted to match the
/// in-app surface — `DWMWA_COLOR_NONE` leaves that strip black.
pub fn apply_platform_window_chrome(app: &tauri::App) {
    #[cfg(any(windows, target_os = "linux"))]
    {
        let Some(win) = app.get_webview_window("main") else {
            return;
        };
        if let Err(e) = win.set_decorations(false) {
            log::warn!("failed to disable native window decorations: {e}");
        }
        if let Err(e) = win.set_shadow(true) {
            log::warn!("failed to enable window shadow: {e}");
        }
        #[cfg(windows)]
        {
            let dark = matches!(win.theme(), Ok(tauri::Theme::Dark));
            apply_windows_chrome(&win, dark, if dark { DARK_BG } else { LIGHT_BG });
        }
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = app;
    }
}

#[cfg(windows)]
fn apply_windows_chrome(win: &tauri::WebviewWindow, dark: bool, bg: &str) {
    let rgb = resolve_surface(bg, dark);
    if let Err(e) = win.set_theme(Some(if dark {
        tauri::Theme::Dark
    } else {
        tauri::Theme::Light
    })) {
        log::warn!("failed to set window theme: {e}");
    }
    if let Err(e) = win.set_background_color(Some(tauri::window::Color(rgb.r, rgb.g, rgb.b, 255))) {
        log::warn!("failed to set window background color: {e}");
    }
    apply_dwm_caption_colors(win, rgb, dark);
}

#[cfg(windows)]
fn apply_dwm_caption_colors(win: &tauri::WebviewWindow, rgb: Rgb, dark: bool) {
    let Ok(hwnd) = win.hwnd() else {
        return;
    };
    // tauri::HWND is windows::Win32::Foundation::HWND (*mut c_void, transparent).
    let raw: *mut std::ffi::c_void = unsafe { std::mem::transmute_copy(&hwnd) };

    const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
    const DWMWA_BORDER_COLOR: u32 = 34;
    const DWMWA_CAPTION_COLOR: u32 = 35;
    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOZORDER: u32 = 0x0004;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const SWP_FRAMECHANGED: u32 = 0x0020;

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: *mut std::ffi::c_void,
            dwattribute: u32,
            pvattribute: *const std::ffi::c_void,
            cbattribute: u32,
        ) -> i32;
    }

    #[link(name = "user32")]
    extern "system" {
        fn SetWindowPos(
            hwnd: *mut std::ffi::c_void,
            hwnd_insert_after: *mut std::ffi::c_void,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            flags: u32,
        ) -> i32;
    }

    unsafe {
        let dark_mode: i32 = i32::from(dark);
        let _ = DwmSetWindowAttribute(
            raw,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark_mode as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
        let color = to_colorref(rgb);
        let _ = DwmSetWindowAttribute(
            raw,
            DWMWA_CAPTION_COLOR,
            (&color as *const u32).cast(),
            std::mem::size_of::<u32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            raw,
            DWMWA_BORDER_COLOR,
            (&color as *const u32).cast(),
            std::mem::size_of::<u32>() as u32,
        );
        let _ = SetWindowPos(
            raw,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

/// Paint the Win11 undecorated-shadow 1px caption strip to match the app surface.
#[tauri::command]
pub fn sync_windows_chrome(app: tauri::AppHandle, dark: bool, bg: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        if let Some(win) = app.get_webview_window("main") {
            apply_windows_chrome(&win, dark, &bg);
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (app, dark, bg);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_rrggbb_and_short() {
        assert_eq!(
            parse_css_color("#ffffff"),
            Some(Rgb {
                r: 255,
                g: 255,
                b: 255
            })
        );
        assert_eq!(
            parse_css_color("#fff"),
            Some(Rgb {
                r: 255,
                g: 255,
                b: 255
            })
        );
        assert_eq!(
            parse_css_color("  #1f2437  "),
            Some(Rgb {
                r: 0x1f,
                g: 0x24,
                b: 0x37
            })
        );
        assert_eq!(
            parse_css_color("#1f2437ff"),
            Some(Rgb {
                r: 0x1f,
                g: 0x24,
                b: 0x37
            })
        );
    }

    #[test]
    fn parse_rgb_function() {
        assert_eq!(
            parse_css_color("rgb(20, 20, 20)"),
            Some(Rgb {
                r: 20,
                g: 20,
                b: 20
            })
        );
        assert_eq!(
            parse_css_color("rgba(255, 0, 128, 0.5)"),
            Some(Rgb {
                r: 255,
                g: 0,
                b: 128
            })
        );
    }

    #[test]
    fn parse_rejects_garbage() {
        assert_eq!(parse_css_color(""), None);
        assert_eq!(parse_css_color("black"), None);
        assert_eq!(parse_css_color("#gg0000"), None);
        assert_eq!(parse_css_color("#12"), None);
    }

    #[test]
    fn colorref_is_bbggrr() {
        assert_eq!(
            to_colorref(Rgb {
                r: 255,
                g: 255,
                b: 255
            }),
            0x00FF_FFFF
        );
        assert_eq!(
            to_colorref(Rgb {
                r: 0x14,
                g: 0x20,
                b: 0x30
            }),
            0x0030_2014
        );
        assert_eq!(
            to_colorref(Rgb {
                r: 0xff,
                g: 0x00,
                b: 0x00
            }),
            0x0000_00FF
        );
    }

    #[test]
    fn resolve_surface_falls_back_by_theme() {
        assert_eq!(
            resolve_surface("nope", false),
            Rgb {
                r: 255,
                g: 255,
                b: 255
            }
        );
        assert_eq!(
            resolve_surface("nope", true),
            Rgb {
                r: 0x1f,
                g: 0x24,
                b: 0x37
            }
        );
        assert_eq!(
            resolve_surface("#009966", true),
            Rgb {
                r: 0x00,
                g: 0x99,
                b: 0x66
            }
        );
    }

    fn window_entry(raw: &str) -> serde_json::Value {
        let v: serde_json::Value = serde_json::from_str(raw).expect("config json");
        v["app"]["windows"][0].clone()
    }

    #[test]
    fn windows_conf_disables_native_frame() {
        let win = window_entry(include_str!("../tauri.windows.conf.json"));
        assert_eq!(win["decorations"], false);
        assert_eq!(win["transparent"], false);
        assert_eq!(win["titleBarStyle"], "Visible");
    }

    #[test]
    fn linux_conf_disables_native_frame() {
        let win = window_entry(include_str!("../tauri.linux.conf.json"));
        assert_eq!(win["decorations"], false);
        assert_eq!(win["transparent"], false);
        assert_eq!(win["titleBarStyle"], "Visible");
    }

    #[test]
    fn macos_base_uses_overlay_without_transparent_frame() {
        let win = window_entry(include_str!("../tauri.conf.json"));
        assert_eq!(win["titleBarStyle"], "Overlay");
        assert_eq!(win["hiddenTitle"], true);
        assert!(win.get("decorations").is_none() || win["decorations"] == true);
        assert_ne!(
            win.get("transparent")
                .cloned()
                .unwrap_or(serde_json::Value::Bool(false)),
            true
        );
    }
}
