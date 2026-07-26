use std::collections::HashMap;
use std::sync::Mutex;

/// Memoized icon extraction for file/shortcut paths. Providers stay unaware
/// of icon work; `commands::search` enriches results after ranking.
pub struct IconCache {
    cache: Mutex<HashMap<String, Option<String>>>,
}

impl IconCache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Returns a `data:image/png;base64,...` URI, or `None` on failure.
    /// Failures are cached so repeated lookups stay cheap.
    pub fn get_or_extract(&self, path: &str) -> Option<String> {
        let mut cache = self.cache.lock().unwrap();
        if let Some(cached) = cache.get(path) {
            return cached.clone();
        }

        let icon = extract_icon(path);
        cache.insert(path.to_string(), icon.clone());
        icon
    }
}

#[cfg(windows)]
fn extract_icon(path: &str) -> Option<String> {
    let base64 = windows_icons::get_icon_base64_by_path(path).ok()?;
    if base64.starts_with("data:") {
        Some(base64)
    } else {
        Some(format!("data:image/png;base64,{base64}"))
    }
}

#[cfg(not(windows))]
fn extract_icon(_path: &str) -> Option<String> {
    None
}
