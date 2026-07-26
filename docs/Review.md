# Review Log

Append-only. Do not edit or remove previous entries - add a new dated entry per work session.

---

## 2026-07-26 - v0.1: Core loop + all 5 providers

### Completed

- Scaffolded the full project: Tauri v2 + React 19 + TypeScript + Tailwind CSS v4 (via `@tailwindcss/vite`), pnpm.
- Installed and verified the Rust toolchain (via `rustup`) and MSVC Build Tools (C++ workload) - neither was present on the machine at session start.
- Built the provider-based `SearchEngine` core: `Provider` trait, `Query`, `SearchResult`/`ResultAction`, parallel fan-out + frecency-adjusted ranking (`src-tauri/src/search/`).
- Implemented all 5 providers: `AppProvider` (Start Menu `.lnk` scan), `CalculatorProvider` (`meval` expression eval), `SettingsProvider` (static `ms-settings:` catalog), `FileProvider` and `FolderProvider` (SQLite/FTS5-backed).
- Built the indexing layer: SQLite schema with FTS5 (trigram tokenizer) + `launch_history` table (`indexer/db.rs`), a background crawler scoped to Desktop/Documents/Downloads/Pictures (`indexer/crawler.rs`), and an incremental file watcher (`indexer/watcher.rs`, `notify` + `notify-debouncer-mini`).
- Wired the window shell: transparent/undecorated pre-warmed window, Mica vibrancy (acrylic fallback), global Alt+Space hotkey toggle, hide-on-focus-loss, top-third positioning, and dynamic auto-resize to content height.
- Built the Apple-like React UI: `SearchBar`, `ResultsList`, `ResultItem`, `ResultIcon`, debounced `useSearch` hook, `useKeyboardNav` (arrows/Enter/Escape), framer-motion selection highlight and fade transitions.
- Verified end-to-end in a real `tauri dev` run: app launched, working set ~47MB, initial index crawl completed in the background without blocking startup, global hotkey toggled the window visible/hidden, and the frosted rounded-card UI rendered correctly positioned in the top third of the screen.

### Files Modified

Entire project created from an empty workspace. Key files:

- `src-tauri/src/lib.rs`, `window.rs`, `commands.rs`
- `src-tauri/src/search/{mod,engine,provider,query,result}.rs`
- `src-tauri/src/providers/{mod,app_provider,calculator_provider,settings_provider,file_provider,folder_provider}.rs`
- `src-tauri/src/indexer/{mod,db,crawler,watcher}.rs`
- `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
- `src/App.tsx`, `src/components/*`, `src/hooks/*`, `src/lib/tauri.ts`, `src/types/result.ts`
- `docs/Architecture.md`, `docs/Decisions.md`, `docs/Roadmap.md` (new)

### Architecture Changes

Established the baseline architecture described in `Architecture.md`: `SearchEngine` <-> `Provider` trait boundary, uniform `SearchResult`/`ResultAction`, SQLite/FTS5-backed indexing layer decoupled from the providers that query it. This is the first session, so this *is* the architecture, not a change to a prior one.

### Performance Impact

- Idle memory: ~47MB working set observed via `Get-Process`, well under the 100MB target.
- Initial index crawl (scoped to Desktop/Documents/Downloads/Pictures) completed in the background within a couple seconds on this dev machine without blocking app startup or the first search.
- Window show/hide is a pure show/focus/hide toggle on a pre-existing window (no webview re-creation), which is what keeps it within the <50ms show-time budget - not yet instrumented with precise timing, see Known Issues.

### Known Issues

- Show-time and search-latency are not yet instrumented with actual timers/logging - the <50ms/<100ms budgets are architecturally targeted but not yet measured in-app. Should add lightweight timing logs behind the debug log level.
- `AppProvider` only scans Win32 Start Menu `.lnk` shortcuts, not UWP/Store apps, and only scans once at startup (no periodic refresh or file-watching on the Start Menu folders).
- No real icon extraction yet - all results use symbolic glyph icons (`ResultIcon` component already supports swapping in real `data:`/`http` icons transparently once extraction is implemented).
- UI is dark-theme only; no light-theme/Windows-theme-adaptive styling yet despite that being called out in the original UI goals.
- Automated keyboard-input testing during this session was done via OS-level `SendKeys`, which is flaky when the IDE/agent's own window is competing for focus - real interactive testing by the user is recommended before considering v0.1 fully verified end-to-end.

### Next Tasks

See `Roadmap.md` v0.2 section - top priorities are real icon extraction, periodic app re-scan, configurable hotkey, and light-theme support.

### Lessons Learned

- This machine had neither Rust nor MSVC Build Tools installed; `winget install ... --override "--add Microsoft.VisualStudio.Workload.VCTools"` silently installed BuildTools *without* the requested workload (no error surfaced), requiring a follow-up `vs_installer setup.exe modify --add Microsoft.VisualStudio.Workload.VCTools` to actually get `cl.exe` present. Worth checking for `cl.exe`/`vswhere -requires VC.Tools` explicitly rather than trusting a winget exit code of 0 as proof the C++ workload landed.
- Tauri's `#[tauri::command]` macro requires all argument/return types to be fully owned (`Deserialize`-safe) - a `&'static str` field on a struct used as a command argument produces a confusing borrow-checker error inside macro-expanded code rather than a clear "type must be owned" message. Worth remembering for any future struct that crosses the IPC boundary.

---

## 2026-07-26 - v0.2: Icons, hotkey, UWP, distribution

### Completed

- Added real icon extraction via `windows-icons` crate and `IconCache` module; `commands::search` enriches `Launch` results with `data:image/png;base64,...` URIs after ranking.
- Extended `AppProvider` to enumerate UWP/Store apps via `PackageManager`, dedupe by name, launch via `shell:AppsFolder\<AUMID>`.
- Added persisted settings (`settings.json` in app data dir) with configurable global hotkey (`get_settings` / `set_hotkey` commands, dynamic unregister/register).
- Added system tray icon (Open Spotlight / Settings / Quit) and a second Settings window with hotkey editor UI (`SettingsApp.tsx`).
- Built release installers successfully: `Spotlight_0.2.0_x64_en-US.msi` and `Spotlight_0.2.0_x64-setup.exe`.
- Documented GitHub Release publishing workflow in README.

### Files Modified

- `src-tauri/src/icons.rs`, `settings.rs`, `hotkey.rs` (new)
- `src-tauri/src/lib.rs`, `commands.rs`, `window.rs`, `providers/app_provider.rs`
- `src-tauri/Cargo.toml`, `tauri.conf.json`, `capabilities/default.json`
- `src/SettingsApp.tsx`, `src/main.tsx`, `src/lib/tauri.ts`, `src/types/settings.ts`
- `vite.config.ts`, `package.json`
- `README.md`, `docs/Architecture.md`, `docs/Decisions.md`, `docs/Roadmap.md`

### Architecture Changes

- Icon enrichment moved to the IPC layer (`commands::search`) rather than providers - keeps the Provider trait boundary clean.
- Settings/hotkey/tray added as orthogonal concerns in `settings.rs`, `hotkey.rs`, and tray setup in `lib.rs`.
- Second webview window (`"settings"`) shares the same Vite bundle; `main.tsx` routes by window label.

### Performance Impact

- Icon extraction is memoized per path in `IconCache` - first lookup per file pays Windows API cost, subsequent keystrokes are free.
- UWP enumeration runs once at startup alongside Start Menu scan; negligible added startup time on dev machine.
- Release binary + installers built successfully; no regression observed in `cargo check`.

### Known Issues

- UWP app icons still use symbolic glyphs (logo extraction from package manifest deferred).
- App list still scanned only once at startup (no periodic refresh).
- Installers are unsigned - Windows SmartScreen will warn on first run.
- Show-time and search-latency still not instrumented with timers.
- Light theme not implemented.

### Next Tasks

See `Roadmap.md` v0.3: UWP icons, periodic app re-scan, indexed-folder picker, light theme, code signing.

### Lessons Learned

- Vite 8 production builds require `esbuild` as an explicit devDependency on this pnpm setup; pnpm 11 also requires `pnpm approve-builds esbuild` before esbuild's postinstall can run.
- When running `pnpm build` outside `tauri build`, set build target to `chrome105` on Windows (`process.platform === 'win32'`) since `TAURI_ENV_PLATFORM` is only set during Tauri builds.

---

## 2026-07-26 - v0.3: Onboarding, settings gear, glass UI, web search

### Completed

- Added `WebSearchProvider` — one result row per non-empty query, opens Google search URL in default browser via `OpenUri`; no network from Spotlight.
- Extended settings with `welcomeDismissed`; added `dismiss_welcome` and `open_settings` commands; auto-show main window on first launch.
- Added first-run welcome strip in `App.tsx` (**Press {hotkey} anytime…** + **Got it**).
- Added settings gear (⚙) on the right of the search bar → Settings window.
- Tuned main card for stronger Apple glass look (`backdrop-blur-3xl`, lighter translucent bg, ring/shadow).

### Files Modified

- `src-tauri/src/providers/web_search_provider.rs` (new)
- `src-tauri/src/settings.rs`, `commands.rs`, `lib.rs`, `providers/mod.rs`, `Cargo.toml`
- `src/App.tsx`, `src/components/SearchBar.tsx`, `src/components/ResultIcon.tsx`
- `src/lib/tauri.ts`, `src/types/settings.ts`
- `docs/Architecture.md`, `docs/Decisions.md`, `docs/Roadmap.md`, `docs/Review.md`

### Architecture Changes

- 6th provider registered: `WebSearchProvider` (score 3.0, `OpenUri` handoff).
- Settings JSON now includes `welcomeDismissed`; first-run auto-show wired in `lib.rs` setup.
- Frontend settings access split: tray menu (existing) + in-bar gear (`open_settings`).

### Performance Impact

- Web provider is pure string formatting — no added I/O or network.
- Welcome strip only renders when query is empty and flag is false; no ongoing cost after dismiss.

### Known Issues

- UWP app icons still use symbolic glyphs.
- App list still scanned only once at startup.
- Installers still unsigned.
- Light theme not implemented.

### Next Tasks

See `Roadmap.md` v0.4: UWP icons, periodic app re-scan, indexed-folder picker, light theme, code signing.

### Lessons Learned

- `#[serde(default)]` on `welcome_dismissed` keeps existing settings.json files valid without migration — older installs pick up `false` automatically.

---

## 2026-07-26 - v0.3.1: Expanded onboarding, tray icon, smooth edges

### Completed

- Expanded first-run welcome strip with compact copy: what Spotlight searches, hotkey, ⚙ settings gear, and taskbar **^** tray hint.
- Added dedicated `tray-icon.png` and updated `setup_tray`: `include_bytes!` icon, tooltip with hotkey, left-click opens Spotlight, right-click menu unchanged.
- Smoothed card edges: CSS compositing on `#root`/`.spotlight-card` + Windows DWM rounded corners in `window.rs`.
- Updated Settings footer to mention both gear and tray paths.

### Files Modified

- `src-tauri/icons/tray-icon.png` (new)
- `src-tauri/src/lib.rs`, `window.rs`, `Cargo.toml`
- `src/App.tsx`, `src/index.css`, `src/SettingsApp.tsx`
- `docs/Architecture.md`, `docs/Review.md`

### Architecture Changes

- Tray no longer uses `default_window_icon()` — dedicated small PNG for Windows overflow visibility.
- Native DWM corner preference applied alongside Mica/acrylic for transparent undecorated window.

### Known Issues

- DWM rounded corners require Windows 11; Windows 10 keeps square native clip (CSS still helps).
- Tray icon still lands in Windows **^** overflow by default until user pins it.

### Lessons Learned

- Windows transparent webviews need painted pixels for click targets and DWM rounding for smooth corners — CSS `border-radius` alone is not enough.
- `Shortcut::from_str` in `tauri-plugin-global-shortcut` makes hotkey parsing trivial - no manual modifier/keycode mapping needed.
