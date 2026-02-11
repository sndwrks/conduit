use crate::config;
use crate::models::Settings;
use crate::state::AppState;
use log::info;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn update_settings(
    settings: Settings,
    state: State<AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let mut current = state.settings.lock().map_err(|e| e.to_string())?;

    // Sync autostart if launch_on_startup changed
    if settings.launch_on_startup != current.launch_on_startup {
        let autolaunch = app.autolaunch();
        if settings.launch_on_startup {
            autolaunch
                .enable()
                .map_err(|e: tauri_plugin_autostart::Error| e.to_string())?;
            info!("Autostart enabled");
        } else {
            autolaunch
                .disable()
                .map_err(|e: tauri_plugin_autostart::Error| e.to_string())?;
            info!("Autostart disabled");
        }
    }

    info!("Settings updated");
    *current = settings.clone();
    config::save_settings(&settings)
}
