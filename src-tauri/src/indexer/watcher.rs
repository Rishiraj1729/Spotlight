use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use notify_debouncer_mini::{new_debouncer, DebounceEventResult, DebouncedEventKind};

use super::crawler::scoped_roots;
use super::db::Db;

/// Watches the scoped roots for changes and incrementally updates the index
/// so newly created/renamed/deleted files show up in search without
/// re-running the (relatively expensive) full crawl.
pub fn spawn_watcher(db: Arc<Db>) {
    std::thread::spawn(move || {
        let db_for_events = db.clone();
        let mut debouncer = match new_debouncer(Duration::from_millis(750), move |result: DebounceEventResult| {
            let Ok(events) = result else { return };
            for event in events {
                if event.kind != DebouncedEventKind::Any {
                    continue;
                }
                handle_path_change(&db_for_events, &event.path);
            }
        }) {
            Ok(d) => d,
            Err(err) => {
                log::warn!("failed to start file watcher: {err}");
                return;
            }
        };

        for root in scoped_roots() {
            if let Err(err) = debouncer.watcher().watch(&root, notify::RecursiveMode::Recursive) {
                log::warn!("failed to watch {}: {err}", root.display());
            }
        }

        // Keep the debouncer (and its background thread) alive for the
        // lifetime of the app.
        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    });
}

fn handle_path_change(db: &Db, path: &std::path::Path) {
    if !path.exists() {
        db.remove_entry(&path.to_string_lossy());
        return;
    }

    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return;
    };
    let is_dir = path.is_dir();
    let modified = path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    db.upsert_entry(&path.to_string_lossy(), name, is_dir, modified);
}
