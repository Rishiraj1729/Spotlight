mod commands;
mod hotkey;
mod icons;
mod indexer;
mod providers;
mod search;
mod settings;
mod window;

use std::sync::Arc;

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use hotkey::HotkeyState;
use icons::IconCache;
use indexer::{crawler, watcher, Db};
use providers::{AppProvider, CalculatorProvider, FileProvider, FolderProvider, SettingsProvider, WebSearchProvider};
use search::SearchEngine;
use settings::{Settings, SettingsStore, DEFAULT_HOTKEY};

fn parse_hotkey(raw: &str) -> Shortcut {
    raw.parse()
        .unwrap_or_else(|_| DEFAULT_HOTKEY.parse().expect("default hotkey must parse"))
}

fn tray_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
}

fn setup_tray(app: &tauri::App, hotkey: &str) -> tauri::Result<()> {
    let open_item = MenuItem::with_id(app, "open", "Open Spotlight", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_item, &settings_item, &quit_item])?;

    let icon = tray_icon()?;
    let tooltip = format!("Spotlight — {hotkey}");

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip(tooltip)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                window::show(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => window::show(app),
            "settings" => window::show_settings(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        window::toggle_window(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let data_dir = app.path().app_data_dir().expect("app data dir must resolve");
            let settings_path = data_dir.join("settings.json");
            let settings = Settings::load(&settings_path);
            let shortcut = parse_hotkey(&settings.hotkey);

            app.manage(SettingsStore::new(settings_path));
            app.manage(HotkeyState::new(shortcut.clone()));
            app.manage(IconCache::new());

            let db = Arc::new(Db::open(&data_dir).expect("failed to open index database"));
            crawler::spawn_initial_crawl(db.clone());
            watcher::spawn_watcher(db.clone());

            let mut engine = SearchEngine::new(db.clone());
            engine.register(Box::new(CalculatorProvider::new()));
            engine.register(Box::new(AppProvider::new()));
            engine.register(Box::new(SettingsProvider::new()));
            engine.register(Box::new(FileProvider::new(db.clone())));
            engine.register(Box::new(FolderProvider::new(db.clone())));
            engine.register(Box::new(WebSearchProvider::new()));
            app.manage(engine);

            if let Some(main_window) = app.get_webview_window(window::MAIN_WINDOW) {
                window::apply_vibrancy(&main_window);

                let dismiss_handle = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let WindowEvent::Focused(false) = event {
                        let _ = dismiss_handle.hide();
                    }
                });
            }

            // The Settings window is created once and reused like the main
            // window - closing via the titlebar X must hide it, not destroy
            // it, or `open_settings` would find nothing on the next open.
            if let Some(settings_window) = app.get_webview_window(window::SETTINGS_WINDOW) {
                let hide_handle = settings_window.clone();
                settings_window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = hide_handle.hide();
                    }
                });
            }

            app.global_shortcut().register(shortcut)?;
            setup_tray(app, &settings.hotkey)?;

            if !settings.welcome_dismissed {
                window::show(app.handle());
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::launch,
            commands::hide_window,
            commands::resize_window,
            commands::get_settings,
            commands::set_hotkey,
            commands::open_settings,
            commands::dismiss_welcome,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
