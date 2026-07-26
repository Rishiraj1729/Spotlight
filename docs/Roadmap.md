# Roadmap

Status legend: `done`, `in progress`, `planned`, `stretch`.

## v0.1 - Core loop with all 5 providers

| Feature | Status | Notes |
|---|---|---|
| Tauri + React/TS/Tailwind scaffold | done | pnpm, Vite, Tailwind v4 via `@tailwindcss/vite` |
| Transparent, undecorated, pre-warmed window | done | Never destroyed; show/hide/focus only |
| Mica/acrylic vibrancy (Windows) | done | Mica with acrylic fallback for older builds |
| Global hotkey (Alt+Space) show/hide | done | `tauri-plugin-global-shortcut` |
| Hide on focus loss / Escape | done | |
| Window auto-resize to content | done | `ResizeObserver` + `resize_window` command |
| Top-third window positioning | done | Recomputed on each show |
| `Provider` trait + `SearchResult`/`Query` structs | done | |
| `SearchEngine` fan-out + ranking + frecency | done | `rayon` parallel dispatch, top-9 truncation |
| App Provider (Start Menu `.lnk` scan) | done | |
| Calculator Provider | done | `meval`-based expression evaluator |
| Settings Provider | done | Static `ms-settings:` catalog, ~28 entries |
| SQLite + FTS5 index (scoped crawl + watcher) | done | Desktop/Documents/Downloads/Pictures |
| File Provider | done | |
| Folder Provider | done | |
| Apple-like UI (SearchBar, ResultsList, keyboard nav) | done | framer-motion selection highlight |
| Documentation (this set of docs) | done | |

## v0.2 - Polish, distribution, and completeness

| Feature | Status | Notes |
|---|---|---|
| Real icon extraction (apps/files via `.lnk`/path) | done | `windows-icons` + `IconCache`; enrichment in `commands::search` |
| UWP app enumeration | done | `PackageManager` merged into `AppProvider` |
| Configurable hotkey | done | JSON persistence + `set_hotkey` command |
| Settings UI (hotkey editor) | done | Second `"settings"` window + tray menu |
| System tray (Open / Settings / Quit) | done | Clean exit path |
| Windows installer build (MSI + NSIS) | done | `pnpm tauri build` -> unsigned bundles |
| GitHub Release publishing docs | done | See README "Releasing" section |

## v0.3 - UX polish and web search

| Feature | Status | Notes |
|---|---|---|
| First-run welcome strip (hotkey hint) | done | `welcomeDismissed` in settings; auto-show on first launch |
| Settings gear in search bar | done | `open_settings` command → Settings window |
| Stronger Apple glass frosted UI | done | `backdrop-blur-3xl`, lighter card bg, ring/shadow tweaks |
| Web search provider (Google in default browser) | done | `WebSearchProvider`, score 3.0, no network from Spotlight |

## v0.4 - Remaining polish (planned)

- UWP app icon extraction (`AppListEntry` logo stream -> PNG base64).
- Periodic re-scan of installed apps (currently scanned once at startup).
- Choose indexed folders in Settings UI.
- Light theme support (UI is currently dark-only).
- Result previews (file thumbnails, folder contents count).
- Persist/restore window position across monitor configuration changes.
- Code signing (remove SmartScreen "unknown publisher" warning).
- winget / Microsoft Store packaging.

## v0.5 - Extensibility (planned)

- Plugin/extension provider loading (third-party providers without recompiling core).
- Bookmarks provider (opt-in, still local-first - no telemetry).
- Snippet/clipboard-history provider.
- Optional Everything SDK backend for File/Folder providers (full-drive search), selectable in settings, without becoming a hard dependency.

## Stretch goals

- Quick actions on results (e.g. "Open containing folder" as a secondary action, right-click menu equivalent).
- Fuzzy math/unit conversion in Calculator Provider (e.g. `10 km to miles`).
- Multi-monitor-aware positioning (currently uses the monitor under the cursor/primary at show-time).
