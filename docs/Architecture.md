# Architecture

## Overview

Spotlight for Windows is a Tauri (Rust backend + React/TypeScript/Tailwind webview frontend) desktop app. The backend owns all search logic, indexing, and OS integration; the frontend is a thin, stateless-as-possible rendering layer over one IPC surface.

```
Global Hotkey (Alt+Space)
        |
        v
  Window Manager  (pre-warmed, transparent, mica/acrylic vibrancy)
        |
        v
     React UI  (SearchBar + ResultsList)
        |  invoke('search', { query })
        v
   SearchEngine (Rust)
        |  fans out to every registered Provider concurrently (rayon)
        v
  ┌───────────┬───────────┬─────────────┬──────────────┬────────────┬──────────────┐
  │AppProvider│FileProvider│FolderProvider│SettingsProvider│CalculatorProvider│WebSearchProvider│
  └───────────┴───────────┴─────────────┴──────────────┴────────────┴──────────────┘
        |  File/Folder providers query
        v
  SQLite index (FTS5, trigram tokenizer)
        ^
        |  kept up to date by
  Background crawler (initial scan) + file watcher (incremental)
```

## The Provider boundary

This is the core architectural rule of the project: **the `SearchEngine` never knows how a provider works internally.**

```rust
pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;
    fn priority(&self) -> f32 { 0.0 }
    fn search(&self, query: &Query) -> Vec<SearchResult>;
}
```

Every provider - regardless of whether it's an in-memory scan (`AppProvider`), a pure computation (`CalculatorProvider`), a static lookup table (`SettingsProvider`), or a SQLite-backed index query (`FileProvider`/`FolderProvider`) - implements this one trait and returns the same `SearchResult` struct. `SearchEngine::search()` (`src-tauri/src/search/engine.rs`) does exactly three things:

1. Fan the query out to every registered provider in parallel (`rayon`).
2. Apply a small frecency bonus per result (usage history lookup).
3. Sort by score and truncate to the top ~9 results.

Adding a 7th provider means writing one new file that implements `Provider` and registering it in `lib.rs` - nothing else in the engine, IPC layer, or frontend changes.

## Data flow: a single keystroke

1. React's `useSearch` hook (`src/hooks/useSearch.ts`) debounces input by 30ms and calls `invoke('search', { query })`.
2. The Rust `search` command (`src-tauri/src/commands.rs`) delegates to `SearchEngine::search`, then optionally enriches `Launch` results with real file icons via `IconCache` (see below).
3. Results (already ranked, capped at 9) are serialized back as `SearchResult[]` and rendered by `ResultsList`.
4. `Enter`/click calls `invoke('launch', { result })`, which pattern-matches the result's `ResultAction` (`Launch`, `OpenUri`, `CopyToClipboard`, `RunSubQuery`) and records the launch for frecency ranking. Web search uses `OpenUri` to hand off a Google search URL to the default browser — no network from Spotlight itself.

## Window lifecycle

The main window is created once, hidden, at app startup (`visible: false` in `tauri.conf.json`) and never destroyed. The global hotkey (registered via `tauri-plugin-global-shortcut`) only shows/hides/focuses it - this is what keeps the "show in <50ms" target achievable, since there's no webview cold start on every toggle. Losing focus (clicking away) also hides the window, matching Spotlight's dismiss behavior.

On first launch (`welcomeDismissed: false` in settings), the main window is shown automatically after tray setup so the in-card welcome strip is visible before the user knows the hotkey. Dismissing the strip via **Got it** calls `dismiss_welcome`, which persists the flag and hides the strip on subsequent opens.

The window auto-resizes to fit its content: it starts collapsed (just the search bar, 680x68), and the frontend reports its rendered height via a `ResizeObserver` + `invoke('resize_window', { height })` every time the result list changes, so the window grows/shrinks like one continuous card instead of a fixed-size list with dead space.

## Indexing (File/Folder providers)

Rather than a full-drive crawl or a dependency on the Everything SDK, the index is:

- **Scoped** to user-relevant folders (Desktop, Documents, Downloads, Pictures) - see `indexer/crawler.rs::scoped_roots()`.
- **Built once in the background** on startup (`crawler::spawn_initial_crawl`), off the UI thread.
- **Kept fresh incrementally** by a debounced filesystem watcher (`indexer/watcher.rs`, `notify` + `notify-debouncer-mini`), so newly created/deleted files show up without a full re-crawl.
- **Queried via SQLite FTS5** with a trigram tokenizer (`indexer/db.rs`), which gives fast substring matching without needing a leading-edge prefix.

This keeps startup indexing time, CPU, and memory low, matching the project's offline-first / low-footprint goals. See [Decisions.md](Decisions.md) for the tradeoffs against Windows Search and the Everything SDK.

## Frecency ranking

A `launch_history` table (`result_id`, `use_count`, `last_used_at`) is updated every time a result is launched. `Db::frecency_bonus()` turns that into a small, capped score boost (max 8 points) blending recency (decays over 30 days) and frequency (caps at 10 uses) - enough to nudge ties toward things you actually use, without ever overriding a clearly better textual match.

## Frontend structure

- `src/lib/tauri.ts` - the only place that calls `invoke()`. Also detects whether we're running inside the Tauri webview (`isTauri()`) and returns mock data otherwise, so the UI can be iterated on in a plain browser via `pnpm dev`.
- `src/hooks/useSearch.ts` - debounced search with stale-response protection (a request counter ignores out-of-order responses from superseded keystrokes).
- `src/hooks/useKeyboardNav.ts` - arrow key navigation, Enter to activate, Escape to dismiss.
- `src/components/` - `SearchBar` (with in-bar settings gear), `ResultsList`, `ResultItem`, `ResultIcon`. All presentational; no business logic.
- `src/SettingsApp.tsx` - settings window UI (hotkey editor), loaded when the `"settings"` webview label is active.
- `src/App.tsx` - first-run welcome strip (reads hotkey from settings, dismissible via `dismiss_welcome`).

## Icon enrichment (v0.2)

Providers return symbolic icon names (`"app"`, `"file"`, etc.). After ranking, `commands::search` enriches results whose action is `Launch { path }` by calling `IconCache::get_or_extract(path)` (`src-tauri/src/icons.rs`), which uses the `windows-icons` crate and memoizes `data:image/png;base64,...` URIs per path. UWP launch targets (`shell:AppsFolder\...`) are skipped for now (package logo extraction is a separate path). This keeps icon I/O out of providers and the search engine.

## App enumeration (v0.2)

`AppProvider` merges two sources at startup:

1. **Win32** - Start Menu `.lnk` shortcuts (existing v0.1 behavior).
2. **UWP** - installed packages via `PackageManager::FindPackagesByUserSecurityId`, launched with `shell:AppsFolder\<AppUserModelId>`.

Duplicates are removed by display name (case-insensitive), preferring the Start Menu entry when both exist.

## Settings, hotkey, and tray (v0.2)

- `settings.rs` persists `{ hotkey, welcomeDismissed }` as JSON in `%AppData%/dev.spotlight.launcher/settings.json`.
- `HotkeyState` tracks the currently registered `Shortcut`; `set_hotkey` unregisters the old combo and registers the new one at runtime.
- A system tray icon (Open Spotlight / Settings / Quit) provides the only clean exit path (main window is `skipTaskbar: true`).
- A second webview window (`"settings"`) hosts `SettingsApp.tsx` for editing the hotkey.

## Web search provider (v0.3)

`WebSearchProvider` returns one fallback row when the query is non-empty: **Search the web for "{query}"** with subtitle **Browser**. The action is `OpenUri { uri: "https://www.google.com/search?q=..." }` — opened via `open::that()` in the user's default browser. Score is fixed at 3.0 so local results rank above it. No HTTP requests from Spotlight.

## Onboarding and in-app settings access (v0.3)

- First-run welcome strip in the main card (when query is empty and `welcomeDismissed` is false) explains what Spotlight does, the hotkey, the ⚙ settings gear, and the taskbar **^** tray menu for quit.
- `open_settings` command opens the Settings window from the search bar gear (⚙) without requiring the tray menu.
- `dismiss_welcome` persists `welcomeDismissed: true`; the backend also auto-shows the main window once on first launch.

## Tray icon and smooth window edges (v0.3.1)

- Dedicated `tray-icon.png` (32×32, high-contrast search glyph) embedded via `include_bytes!` in `setup_tray`; tooltip shows `Spotlight — {hotkey}`. Left-click opens Spotlight; right-click shows the menu (Open / Settings / Quit).
- Main window corners: CSS clip/isolation on `#root` + `.spotlight-card`, plus Windows 11 `DWMWA_WINDOW_CORNER_PREFERENCE` / `DWMWA_BORDER_COLOR` in `window.rs` to avoid jagged transparent-window edges.

## Distribution (v0.2)

`pnpm tauri build` produces unsigned Windows installers:

- `src-tauri/target/release/bundle/msi/Spotlight_0.2.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/Spotlight_0.2.0_x64-setup.exe`

These can be attached to a GitHub Release for others to download. Unsigned builds trigger Windows SmartScreen on first run until code-signed (future work).
