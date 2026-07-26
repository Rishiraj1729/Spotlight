# Decisions

Append-only log of technical decisions and the reasoning behind them. Newest entries at the bottom of each dated section; do not edit or remove past entries.

---

## 2026-07-26 - Tauri v2 over Electron / raw Win32

**Decision:** Build on Tauri v2 (Rust backend + system webview) rather than Electron or a hand-rolled Win32/WinUI app.

**Why:** The performance goals (search window <50ms, <100MB memory, minimal idle CPU) rule out Electron's bundled Chromium overhead. Tauri uses the OS-provided WebView2 runtime on Windows, keeping binary size and idle memory far lower, while still letting us use React/Tailwind for the UI instead of hand-rolling WinUI/XAML. Raw Win32 would give the most control but would slow iteration on the UI significantly and conflicts with "keep modules small, avoid unnecessary abstraction" - Tauri's IPC boundary already gives us a clean Rust/frontend split for free.

## 2026-07-26 - Provider-based SearchEngine architecture

**Decision:** `SearchEngine` only depends on a `Provider` trait and a uniform `SearchResult` struct; it has zero knowledge of how any individual provider finds its data.

**Why:** This is the architectural requirement from the project brief, and it pays off immediately: Calculator (pure computation), Settings (static lookup table), App (in-memory scan), and File/Folder (SQLite-backed) providers all have wildly different internals but plug into the exact same fan-out/rank/truncate pipeline. Adding a 6th provider later (e.g. a Web Search provider or a Snippets provider) requires zero changes to the engine or IPC layer.

## 2026-07-26 - Custom SQLite index (scoped crawl) over Windows Search Index or Everything SDK

**Decision:** File/Folder providers are backed by a custom SQLite + FTS5 index, built by a background crawler scoped to Desktop/Documents/Downloads/Pictures (not a full-drive crawl), kept fresh by a filesystem watcher.

**Why (tradeoffs considered):**
- **Windows Search Index** would mean querying an index we don't control and can't guarantee is enabled/fully built on the user's machine, and it pulls in COM/Search API complexity for a benefit (mostly speed) we can get ourselves at a smaller scope.
- **Everything SDK** offers near-instant full-drive search backed by the MFT, but it's an optional third-party runtime dependency (violates "everything local/offline-first" as a hard requirement without user opt-in) and would need to be bundled or require a separate install.
- **Custom scoped SQLite index** keeps everything in-process, uses only Rust crates we already vendor, and staying scoped to user-relevant folders keeps initial crawl time, memory, and CPU low - directly serving the "Minimal CPU usage while idle" and "<100MB memory" goals. It's slower/less complete than Everything for power users who want to search `C:\` entirely, so that remains a natural, clearly-separated future option (same `Provider` interface, swappable backend) rather than something we need to solve on day one.

## 2026-07-26 - Window created once, hidden, never destroyed

**Decision:** The main window is created at app startup with `visible: false` and is only shown/hidden/focused thereafter - never re-created or destroyed.

**Why:** WebView2 cold start (navigation, JS engine init, first paint) is far slower than the 50ms show-time budget allows. Pre-warming the window once at startup and treating the hotkey purely as a show/hide/focus toggle is the only way to reliably hit that budget on every subsequent invocation.

## 2026-07-26 - Window auto-resizes to content instead of a fixed-size result panel

**Decision:** The window starts collapsed to just the search bar (680x68) and grows to fit the result list via a frontend `ResizeObserver` calling a `resize_window` command, rather than allocating a fixed-size window with an empty/dead area below short result lists.

**Why:** Matches the requested Apple-like feel (Spotlight's window visibly grows/shrinks with content) and avoids a transparent dead zone intercepting clicks with nothing to show for it.

## 2026-07-26 - `SearchResult.provider` is an owned `String`, not `&'static str`

**Decision:** Despite provider IDs being static string literals internally, the field crossing the Tauri IPC boundary is `String`.

**Why:** `SearchResult` is deserialized from the frontend on `launch` (Tauri commands generate a `Deserialize` impl for command arguments). A `&'static str` field cannot be safely produced by an owned-data `Deserialize` impl - the Rust compiler correctly rejects it (borrow-checker lifetime error surfaced through the `tauri::command` macro). Each `Provider::id()` still returns `&'static str` for zero-cost internal use; it's only converted to an owned `String` at the point a `SearchResult` is constructed.

## 2026-07-26 - `windows-icons` crate over hand-rolled GDI FFI

**Decision:** Use the `windows-icons` crate for file/shortcut icon extraction instead of calling `SHGetFileInfoW` / `GetDIBits` directly.

**Why:** Same end result (PNG base64 for the webview), far less FFI surface area and fewer ways to leak GDI handles. Enrichment happens in `commands::search` after ranking so providers stay unaware of icon I/O.

## 2026-07-26 - UWP apps folded into `AppProvider`, not a 6th provider

**Decision:** Enumerate UWP/Store apps inside `AppProvider` alongside Start Menu `.lnk` shortcuts, deduping by display name.

**Why:** From the user's perspective both are "installed applications" with the same launch UX. A separate provider would duplicate ranking logic and split app results artificially. Launch uses `shell:AppsFolder\<AUMID>` via the existing `Launch` action path.

## 2026-07-26 - Tray icon + Settings window over config-file-only hotkey editing

**Decision:** Add a system tray menu (Open / Settings / Quit) and a small Settings webview window for changing the hotkey, backed by JSON persistence.

**Why:** A hand-edited config file is unfriendly and doesn't solve the "no way to quit" problem (main window is hidden from the taskbar). Tray + Settings matches how background launcher utilities behave on Windows and gives a clear error surface when a hotkey is already taken by another app.

## 2026-07-26 - In-card welcome strip over a separate onboarding wizard

**Decision:** Show a dismissible welcome strip inside the main Spotlight card on first run (when `welcomeDismissed` is false), and auto-open the main window once at startup so the user sees it without knowing the hotkey yet.

**Why:** A separate wizard window adds another webview lifecycle and feels heavier than Spotlight's own minimal first-use hint. The strip reads the actual configured hotkey from settings (not a hardcoded string), dismisses with one click, and persists via `dismiss_welcome` — no modal flow or extra navigation.

## 2026-07-26 - Settings gear in search bar alongside tray menu

**Decision:** Add a ⚙ button on the right side of the search bar that calls `open_settings`, in addition to the existing tray → Settings path.

**Why:** Tray-only settings access is easy to miss for first-time users who haven't discovered the tray icon yet. The gear is discoverable without duplicating settings UI — it opens the same Settings window. `tabIndex={-1}` keeps keyboard navigation focused on results.

## 2026-07-26 - Web search as a 6th provider (Google URL handoff)

**Decision:** Add `WebSearchProvider` that returns one row per non-empty query, opening `https://www.google.com/search?q=...` via `OpenUri` in the default browser. No inline results, no API calls from Spotlight.

**Why:** Matches the project's privacy/offline-first stance (Spotlight itself stays local) while giving users a familiar escape hatch when local results aren't enough. Fixed score 3.0 keeps it below indexed/local matches. Same `Provider` boundary as every other source — no special-case in the engine.
