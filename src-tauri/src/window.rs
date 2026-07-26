use tauri::{AppHandle, Manager, WebviewWindow};

pub const MAIN_WINDOW: &str = "main";
pub const SETTINGS_WINDOW: &str = "settings";
pub const COLLAPSED_HEIGHT: f64 = 68.0;

/// Applies the frosted-glass look. Mica reads the desktop wallpaper/theme
/// (closest Windows equivalent to macOS vibrancy); we fall back to acrylic
/// on older Windows builds where Mica isn't available.
#[cfg(target_os = "windows")]
pub fn apply_vibrancy(window: &WebviewWindow) {
    if window_vibrancy::apply_mica(window, None).is_err() {
        let _ = window_vibrancy::apply_acrylic(window, Some((8, 8, 10, 175)));
    }
    apply_rounded_corners(window);
}

#[cfg(not(target_os = "windows"))]
pub fn apply_vibrancy(_window: &WebviewWindow) {}

/// Rounds the native window clip on Windows 11 so CSS border-radius doesn't
/// leave jagged stair-stepped corners on transparent webviews.
#[cfg(target_os = "windows")]
fn apply_rounded_corners(window: &WebviewWindow) {
    use std::mem;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };

    const DWMWA_COLOR_NONE: u32 = 0xFFFF_FFFE;

    let Ok(tauri_hwnd) = window.hwnd() else {
        return;
    };
    // Re-wrap the raw handle so it matches our `windows` crate version.
    let hwnd = HWND(tauri_hwnd.0);

    let round = DWMWCP_ROUND;
    let none_color = DWMWA_COLOR_NONE;

    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &round as *const _ as *const _,
            mem::size_of_val(&round) as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &none_color as *const _ as *const _,
            mem::size_of_val(&none_color) as u32,
        );
    }
}

/// Shows the window centered and focused, or hides it if already visible -
/// the classic Spotlight toggle behavior bound to the global hotkey.
/// The window is created once at startup and never destroyed, so this is
/// just a cheap show/hide + focus, keeping us under the 50ms show budget.
pub fn toggle_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };

    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
        let _ = window.hide();
    } else {
        let _ = window.set_size(tauri::LogicalSize::new(680.0, COLLAPSED_HEIGHT));
        position_top_third(&window);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Centers the window horizontally and places it about a fifth of the way
/// down the screen - the classic Spotlight position, rather than a plain
/// dead-center window.
fn position_top_third(window: &WebviewWindow) {
    let Ok(Some(monitor)) = window.current_monitor() else {
        let _ = window.center();
        return;
    };
    let Ok(window_size) = window.outer_size() else {
        let _ = window.center();
        return;
    };

    let monitor_size = monitor.size();
    let x = (monitor_size.width as f64 - window_size.width as f64) / 2.0;
    let y = monitor_size.height as f64 * 0.2;

    let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.hide();
    }
}

/// Always shows and focuses the main Spotlight window (used by the tray menu).
pub fn show(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };

    let _ = window.set_size(tauri::LogicalSize::new(680.0, COLLAPSED_HEIGHT));
    position_top_third(&window);
    let _ = window.show();
    let _ = window.set_focus();
}

/// Shows and focuses the Settings window above other apps.
pub fn show_settings(app: &AppHandle) {
    let Some(window) = app.get_webview_window(SETTINGS_WINDOW) else {
        return;
    };

    let _ = window.center();
    let _ = window.set_always_on_top(true);
    let _ = window.show();
    let _ = window.set_focus();
}
