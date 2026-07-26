use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

pub const DEFAULT_HOTKEY: &str = "Alt+Space";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub hotkey: String,
    #[serde(default)]
    pub welcome_dismissed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: DEFAULT_HOTKEY.to_string(),
            welcome_dismissed: false,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}

/// Thread-safe access to persisted settings on disk.
pub struct SettingsStore {
    path: PathBuf,
    inner: Mutex<Settings>,
}

impl SettingsStore {
    pub fn new(path: PathBuf) -> Self {
        let settings = Settings::load(&path);
        Self {
            path,
            inner: Mutex::new(settings),
        }
    }

    pub fn get(&self) -> Settings {
        self.inner.lock().unwrap().clone()
    }

    pub fn update_hotkey(&self, hotkey: &str) -> Result<(), String> {
        let mut settings = self.inner.lock().unwrap();
        settings.hotkey = hotkey.to_string();
        settings.save(&self.path)
    }

    pub fn dismiss_welcome(&self) -> Result<(), String> {
        let mut settings = self.inner.lock().unwrap();
        settings.welcome_dismissed = true;
        settings.save(&self.path)
    }
}
