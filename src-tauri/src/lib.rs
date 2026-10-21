mod app_state;
mod commands;
mod capture;
mod audio;
mod replay_buffer;
mod encoding;
mod export;
mod library;
mod storage;
mod settings;
mod shortcuts;
mod diagnostics;

use tauri::{
    Manager,
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    menu::{Menu, MenuItem, PredefinedMenuItem},
};
use std::sync::Arc;
use tracing::info;

use app_state::AppState;
use settings::config::{Settings, ShortcutSettings};
use replay_buffer::engine::ReplayEngine;
use library::manager::Library;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()
                .expect("Could not get app data directory");
            let log_dir = data_dir.join("logs");

            if let Err(e) = diagnostics::init_logging(&log_dir) {
                eprintln!("Failed to init logging: {}", e);
            }
            info!("Open Replay starting. Data dir: {:?}", data_dir);

            let mut settings = Settings::load(&data_dir);
            settings.validate();
            let shortcuts = settings.shortcuts.clone();

            let library = Library::new(&data_dir)
                .expect("Failed to initialize clip library");

            let settings_arc = Arc::new(parking_lot::Mutex::new(settings.clone()));
            let replay_engine = ReplayEngine::new(settings_arc);

            let auto_start = settings.replay.auto_start && settings.replay.enabled;
            let state = AppState::new(replay_engine, library, settings);
            app.manage(state);

            setup_tray(app)?;
            register_shortcuts(app.handle(), &shortcuts)
                .map_err(std::io::Error::other)?;

            if auto_start {
                let s: tauri::State<AppState> = app.state();
                let mut engine = s.replay_engine.lock().unwrap();
                if let Err(e) = engine.start() {
                    tracing::warn!("Auto-start failed: {}", e);
                }
            }

            info!("Open Replay initialized");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_replay,
            commands::stop_replay,
            commands::save_replay,
            commands::get_replay_status,
            commands::get_available_monitors,
            commands::get_available_audio_devices,
            commands::get_encoder_capabilities,
            commands::get_clips,
            commands::get_clip,
            commands::delete_clip,
            commands::rename_clip,
            commands::open_clip_folder,
            commands::get_clip_thumbnail,
            commands::get_settings,
            commands::update_settings,
            commands::get_storage_info,
            commands::start_export,
            commands::cancel_export,
            commands::get_export_progress,
            commands::get_system_info,
            commands::open_logs_folder,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Open Replay");
}

fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Open Replay", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "toggle_replay", "Toggle Replay", true, None::<&str>)?;
    let save = MenuItem::with_id(app, "save_clip", "Save Clip Now", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &toggle, &save, &sep, &quit])?;

    TrayIconBuilder::with_id("main-tray")
        .tooltip("Open Replay")
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show(); let _ = w.set_focus();
                }
            }
            "toggle_replay" => {
                let s: tauri::State<AppState> = app.state();
                let mut engine = s.replay_engine.lock().unwrap();
                if engine.is_recording() { engine.stop(); } else { let _ = engine.start(); }
            }
            "save_clip" => {
                let s: tauri::State<AppState> = app.state();
                let path = { let lib = s.library.lock().unwrap(); lib.next_clip_path() };
                let result = { let engine = s.replay_engine.lock().unwrap(); engine.save_replay(&path) };
                if let Ok((p, dur, w, h, fps, audio)) = result {
                    let lib = s.library.lock().unwrap();
                    let _ = lib.add_clip(&p, dur, w, h, fps, audio);
                }
            }
            "quit" => {
                let s: tauri::State<AppState> = app.state();
                s.replay_engine.lock().unwrap().stop();
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up, ..
            } = event {
                if let Some(w) = tray.app_handle().get_webview_window("main") {
                    let _ = w.show(); let _ = w.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

pub(crate) fn register_shortcuts(
    app: &tauri::AppHandle,
    shortcuts: &ShortcutSettings,
) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
    use std::str::FromStr;

    #[derive(Clone, Copy)]
    enum Action { SaveClip, ToggleReplay, ShowWindow }

    let definitions = [
        (shortcuts.save_replay.as_str(), Action::SaveClip),
        (shortcuts.toggle_replay.as_str(), Action::ToggleReplay),
        (shortcuts.show_window.as_str(), Action::ShowWindow),
    ];
    let parsed = definitions
        .into_iter()
        .map(|(value, action)| {
            Shortcut::from_str(value)
                .map(|shortcut| (shortcut, action))
                .map_err(|e| format!("Invalid shortcut '{value}': {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;

    for (shortcut, action) in parsed {
        app.global_shortcut()
            .on_shortcut(shortcut, move |app, _, event| {
                if event.state != ShortcutState::Pressed { return; }
                match action {
                    Action::SaveClip => {
                        let state: tauri::State<AppState> = app.state();
                        let path = { state.library.lock().unwrap().next_clip_path() };
                        let result = {
                            state.replay_engine.lock().unwrap().save_replay(&path)
                        };
                        match result {
                            Ok((path, duration, width, height, fps, audio)) => {
                                let library = state.library.lock().unwrap();
                                if let Err(error) = library.add_clip(
                                    &path, duration, width, height, fps, audio,
                                ) {
                                    tracing::warn!("Hotkey clip registration failed: {error}");
                                } else {
                                    info!("Replay saved via hotkey");
                                }
                            }
                            Err(error) => tracing::warn!("Hotkey save failed: {error}"),
                        }
                    }
                    Action::ToggleReplay => {
                        let state: tauri::State<AppState> = app.state();
                        let mut engine = state.replay_engine.lock().unwrap();
                        if engine.is_recording() { engine.stop(); }
                        else if let Err(error) = engine.start() {
                            tracing::warn!("Hotkey toggle failed: {error}");
                        }
                    }
                    Action::ShowWindow => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            })
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
