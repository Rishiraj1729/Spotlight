use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::search::{Provider, Query, ResultAction, SearchResult};

/// Static catalog of the most commonly reached-for Windows settings pages.
/// Mapped to `ms-settings:` URIs, which the OS resolves directly - no need
/// to shell out to control panel applets. Extend this list over time; it's
/// intentionally a flat, easy-to-edit table rather than an abstraction.
const SETTINGS_CATALOG: &[(&str, &str)] = &[
    ("Display settings", "ms-settings:display"),
    ("Sound settings", "ms-settings:sound"),
    ("Notifications", "ms-settings:notifications"),
    ("Bluetooth & devices", "ms-settings:bluetooth"),
    ("Network & internet", "ms-settings:network"),
    ("Wi-Fi", "ms-settings:network-wifi"),
    ("Personalization", "ms-settings:personalization"),
    ("Background", "ms-settings:personalization-background"),
    ("Apps & features", "ms-settings:appsfeatures"),
    ("Default apps", "ms-settings:defaultapps"),
    ("Accounts", "ms-settings:accounts"),
    ("Windows Update", "ms-settings:windowsupdate"),
    ("Storage", "ms-settings:storagesense"),
    ("Battery", "ms-settings:batterysaver"),
    ("Power & sleep", "ms-settings:powersleep"),
    ("Privacy & security", "ms-settings:privacy"),
    ("Date & time", "ms-settings:dateandtime"),
    ("Language & region", "ms-settings:regionlanguage"),
    ("Multitasking", "ms-settings:multitasking"),
    ("Taskbar", "ms-settings:taskbar"),
    ("Focus assist", "ms-settings:quiethours"),
    ("Accessibility", "ms-settings:easeofaccess"),
    ("Mouse", "ms-settings:mousetouchpad"),
    ("Keyboard", "ms-settings:keyboard"),
    ("Display resolution", "ms-settings:display-advanced"),
    ("Night light", "ms-settings:nightlight"),
    ("Firewall", "ms-settings:windowsdefender"),
    ("About this PC", "ms-settings:about"),
    ("Control Panel", "control"),
];

pub struct SettingsProvider {
    matcher: SkimMatcherV2,
}

impl SettingsProvider {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }
}

impl Provider for SettingsProvider {
    fn id(&self) -> &'static str {
        "settings"
    }

    fn priority(&self) -> f32 {
        10.0
    }

    fn search(&self, query: &Query) -> Vec<SearchResult> {
        let input = query.trimmed();
        let mut results: Vec<SearchResult> = SETTINGS_CATALOG
            .iter()
            .filter_map(|(name, uri)| {
                let score = self.matcher.fuzzy_match(name, input)?;
                Some(SearchResult {
                    id: format!("settings:{uri}"),
                    provider: self.id().to_string(),
                    title: name.to_string(),
                    subtitle: Some("Setting".to_string()),
                    icon: "settings".to_string(),
                    score: score as f32 + self.priority(),
                    action: ResultAction::OpenUri { uri: uri.to_string() },
                })
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(5);
        results
    }
}
