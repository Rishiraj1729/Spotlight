use std::collections::HashSet;
use std::path::PathBuf;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use walkdir::WalkDir;

use crate::search::{Provider, Query, ResultAction, SearchResult};

#[derive(Debug, Clone)]
struct AppEntry {
    name: String,
    /// Launch target: a Start Menu `.lnk` path, or `shell:AppsFolder\...` for UWP.
    path: String,
}

/// Enumerates installed applications from Start Menu shortcuts and UWP packages.
/// Scanned once at startup and kept in memory.
pub struct AppProvider {
    apps: Vec<AppEntry>,
    matcher: SkimMatcherV2,
}

fn start_menu_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(appdata) = dirs::data_dir() {
        roots.push(appdata.join("Microsoft/Windows/Start Menu/Programs"));
    }
    roots.push(PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs"));
    roots
}

fn scan_start_menu_apps() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    for root in start_menu_roots() {
        for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            let is_shortcut = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("lnk"))
                .unwrap_or(false);
            if !is_shortcut {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            apps.push(AppEntry {
                name: stem.to_string(),
                path: path.to_string_lossy().to_string(),
            });
        }
    }
    apps
}

#[cfg(windows)]
fn scan_uwp_apps() -> Vec<AppEntry> {
    use windows::core::HSTRING;
    use windows::Management::Deployment::PackageManager;

    let mut apps = Vec::new();
    let Ok(pm) = PackageManager::new() else {
        return apps;
    };
    let Ok(packages) = pm.FindPackagesByUserSecurityId(&HSTRING::new()) else {
        return apps;
    };

    for package in packages {
        let Ok(entries) = package.GetAppListEntries() else {
            continue;
        };
        for entry in entries {
            let Ok(display_info) = entry.DisplayInfo() else {
                continue;
            };
            let Ok(aumid) = entry.AppUserModelId() else {
                continue;
            };
            let Ok(name) = display_info.DisplayName() else {
                continue;
            };
            let name = name.to_string();
            if name.is_empty() {
                continue;
            }
            apps.push(AppEntry {
                name,
                path: format!("shell:AppsFolder\\{aumid}"),
            });
        }
    }

    apps
}

#[cfg(not(windows))]
fn scan_uwp_apps() -> Vec<AppEntry> {
    Vec::new()
}

fn merge_apps(start_menu: Vec<AppEntry>, uwp: Vec<AppEntry>) -> Vec<AppEntry> {
    let mut seen = HashSet::new();
    let mut apps = Vec::new();

    for app in start_menu.into_iter().chain(uwp) {
        let key = app.name.to_lowercase();
        if seen.insert(key) {
            apps.push(app);
        }
    }

    apps
}

impl AppProvider {
    pub fn new() -> Self {
        let start_menu = scan_start_menu_apps();
        let uwp = scan_uwp_apps();
        Self {
            apps: merge_apps(start_menu, uwp),
            matcher: SkimMatcherV2::default(),
        }
    }
}

impl Provider for AppProvider {
    fn id(&self) -> &'static str {
        "app"
    }

    fn priority(&self) -> f32 {
        30.0
    }

    fn search(&self, query: &Query) -> Vec<SearchResult> {
        let input = query.trimmed();
        let mut results: Vec<SearchResult> = self
            .apps
            .iter()
            .filter_map(|app| {
                let score = self.matcher.fuzzy_match(&app.name, input)?;
                Some(SearchResult {
                    id: format!("app:{}", app.path),
                    provider: self.id().to_string(),
                    title: app.name.clone(),
                    subtitle: Some("Application".to_string()),
                    icon: "app".to_string(),
                    score: score as f32 + self.priority(),
                    action: ResultAction::Launch { path: app.path.clone() },
                })
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(8);
        results
    }
}
