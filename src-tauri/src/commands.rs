use tauri::{AppHandle, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::hotkey::HotkeyState;
use crate::icons::IconCache;
use crate::search::{ResultAction, SearchEngine, SearchResult};
use crate::settings::{Settings, SettingsStore};
use crate::window::{self, MAIN_WINDOW};

/// The single entry point the frontend calls on every keystroke. All the
/// fan-out/ranking complexity lives in `SearchEngine` - this command is
/// just IPC plumbing, plus optional icon enrichment afterward.
#[tauri::command]
pub fn search(
    engine: State<'_, SearchEngine>,
    icons: State<'_, IconCache>,
    query: String,
) -> Vec<SearchResult> {
    let mut results = engine.search(&query);

    for result in &mut results {
        if let ResultAction::Launch { path } = &result.action {
            // UWP icons need a separate extraction path; keep symbolic glyph.
            if path.starts_with("shell:AppsFolder") {
                continue;
            }
            if let Some(data_uri) = icons.get_or_extract(path) {
                result.icon = data_uri;
            }
        }
    }

    results
}

/// Executes a result's action and records it for frecency ranking, then the
/// frontend hides the window on success.
#[tauri::command]
pub fn launch(engine: State<'_, SearchEngine>, result: SearchResult) -> Result<(), String> {
    match result.action {
        ResultAction::Launch { path } => {
            open::that(&path).map_err(|e| e.to_string())?;
        }
        ResultAction::OpenUri { uri } => {
            open::that(&uri).map_err(|e| e.to_string())?;
        }
        ResultAction::CopyToClipboard { text } => {
            let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
            clipboard.set_text(text).map_err(|e| e.to_string())?;
        }
        ResultAction::RunSubQuery { .. } => {
            // Reserved for future provider drill-down; no-op for v0.1.
        }
    }

    engine.record_launch(&result.id, &result.provider);
    Ok(())
}

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    window::hide(&app);
}

/// Lets the window grow/shrink to fit the current result list, so the UI
/// reads as one continuous card rather than a fixed-size list with empty
/// space - matching the macOS Spotlight feel.
#[tauri::command]
pub fn resize_window(app: AppHandle, height: f64) {
    if let Some(win) = app.get_webview_window(MAIN_WINDOW) {
        let clamped = height.max(window::COLLAPSED_HEIGHT);
        let _ = win.set_size(tauri::LogicalSize::new(680.0, clamped));
    }
}

#[tauri::command]
pub fn get_settings(store: State<'_, SettingsStore>) -> Settings {
    store.get()
}

#[tauri::command]
pub fn set_hotkey(
    app: AppHandle,
    hotkey_state: State<'_, HotkeyState>,
    store: State<'_, SettingsStore>,
    combo: String,
) -> Result<(), String> {
    let trimmed = combo.trim();
    if trimmed.is_empty() {
        return Err("Hotkey cannot be empty".to_string());
    }

    let new_shortcut: Shortcut = trimmed
        .parse()
        .map_err(|_| format!("Invalid hotkey format: {trimmed}"))?;

    let mut current = hotkey_state.shortcut.lock().unwrap();
    let _ = app.global_shortcut().unregister(*current);

    app.global_shortcut()
        .register(new_shortcut.clone())
        .map_err(|e| format!("Could not register hotkey (may be in use): {e}"))?;

    *current = new_shortcut;
    store.update_hotkey(trimmed)?;
    Ok(())
}

#[tauri::command]
pub fn open_settings(app: AppHandle) {
    window::show_settings(&app);
}

#[tauri::command]
pub fn dismiss_welcome(store: State<'_, SettingsStore>) -> Result<(), String> {
    store.dismiss_welcome()
}
