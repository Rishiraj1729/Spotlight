use std::sync::Mutex;

use tauri_plugin_global_shortcut::Shortcut;

/// Tracks the currently registered global shortcut so it can be swapped at
/// runtime when the user changes it in Settings.
pub struct HotkeyState {
    pub shortcut: Mutex<Shortcut>,
}

impl HotkeyState {
    pub fn new(shortcut: Shortcut) -> Self {
        Self {
            shortcut: Mutex::new(shortcut),
        }
    }
}
