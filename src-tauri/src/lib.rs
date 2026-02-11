mod commands;
mod config;
mod midi_engine;
mod models;
mod osc_engine;
mod router;
mod state;

use log::info;
use state::{AppState, EngineHandle};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_log::{Target, TargetKind, TimezoneStrategy, RotationStrategy};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings = config::load_settings().unwrap_or_default();
    let mappings = config::load_mappings().unwrap_or_default();

    let launch_on_startup = settings.launch_on_startup;
    let mapping_count = mappings.len();

    let app_state = AppState {
        settings: Arc::new(Mutex::new(settings)),
        mappings: Arc::new(Mutex::new(mappings)),
        engine: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .max_file_size(5_000_000)
                .rotation_strategy(RotationStrategy::KeepOne)
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::mappings::get_mappings,
            commands::mappings::add_mapping,
            commands::mappings::update_mapping,
            commands::mappings::delete_mapping,
            commands::mappings::reorder_mappings,
            commands::midi::list_midi_inputs,
            commands::midi::list_midi_outputs,
            commands::engine::start_engine,
            commands::engine::stop_engine,
            commands::engine::get_engine_status,
        ])
        .setup(move |app| {
            info!("Conduit starting — {} mappings loaded", mapping_count);

            // Sync autostart state from persisted settings
            let autolaunch = app.autolaunch();
            let is_enabled = autolaunch.is_enabled().unwrap_or(false);
            if launch_on_startup != is_enabled {
                if launch_on_startup {
                    let _ = autolaunch.enable();
                    info!("Autostart enabled (synced from settings)");
                } else {
                    let _ = autolaunch.disable();
                    info!("Autostart disabled (synced from settings)");
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app = window.app_handle();
                let state: tauri::State<AppState> = app.state();
                // Stop engine on close
                {
                    if let Ok(mut engine_guard) = state.engine.lock() {
                        let handle: Option<EngineHandle> = engine_guard.take();
                        if let Some(h) = handle {
                            h.cancel_token.cancel();
                        }
                    }
                }
                // Flush settings
                {
                    if let Ok(settings) = state.settings.lock() {
                        let _ = config::save_settings(&settings);
                    }
                }
                drop(state);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
