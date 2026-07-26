use std::path::PathBuf;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use walkdir::WalkDir;

use super::db::Db;

/// Directory names we never want to descend into: noisy, huge, or
/// irrelevant to "things a person is looking for" - keeps the crawl fast
/// and the index small (memory/perf goals).
const SKIP_DIR_NAMES: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "$RECYCLE.BIN",
    "System Volume Information",
    ".cache",
    ".venv",
    "venv",
    "__pycache__",
];

/// Deliberately scoped to user-relevant folders rather than a full-drive
/// crawl - keeps initial indexing time, CPU and memory low, in line with
/// the project's offline-first/low-footprint goals. Everything SDK remains
/// a pluggable option for a later "index everything" mode.
pub fn scoped_roots() -> Vec<PathBuf> {
    [
        dirs::desktop_dir(),
        dirs::document_dir(),
        dirs::download_dir(),
        dirs::picture_dir(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn should_skip(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|name| name.starts_with('.') || SKIP_DIR_NAMES.contains(&name))
        .unwrap_or(false)
}

/// Runs the initial full crawl of the scoped roots on a background thread
/// so it never blocks app startup or the UI thread.
pub fn spawn_initial_crawl(db: Arc<Db>) {
    std::thread::spawn(move || {
        for root in scoped_roots() {
            crawl_root(&db, &root);
        }
        log::info!("initial index crawl complete");
    });
}

fn crawl_root(db: &Db, root: &PathBuf) {
    let walker = WalkDir::new(root).max_depth(8).into_iter().filter_entry(|e| !should_skip(e));

    for entry in walker.filter_map(Result::ok) {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let is_dir = entry.file_type().is_dir();
        let modified = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        db.upsert_entry(&path.to_string_lossy(), name, is_dir, modified);
    }
}
