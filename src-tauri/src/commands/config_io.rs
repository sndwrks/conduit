use crate::config;
use crate::models::{Mapping, Settings};
use crate::state::AppState;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize, Deserialize)]
struct ConfigExport {
    version: String,
    settings: Settings,
    mappings: Vec<Mapping>,
}

#[tauri::command]
pub async fn export_config(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let mappings = state.mappings.lock().map_err(|e| e.to_string())?.clone();

    let export = ConfigExport {
        version: "1".to_string(),
        settings,
        mappings,
    };

    let json = serde_json::to_string_pretty(&export)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    let path = app
        .dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name("conduit-config.json")
        .blocking_save_file();

    match path {
        Some(file_path) => {
            let path = file_path.as_path().ok_or("Invalid file path")?;
            fs::write(path, json).map_err(|e| {
                error!("Failed to write config export: {}", e);
                format!("Failed to write file: {}", e)
            })?;
            info!("Config exported to {:?}", path);
            Ok(())
        }
        None => {
            // User cancelled the dialog
            Ok(())
        }
    }
}

#[tauri::command]
pub async fn import_config(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<bool, String> {
    let path = app
        .dialog()
        .file()
        .add_filter("JSON", &["json"])
        .blocking_pick_file();

    match path {
        Some(file_path) => {
            let path = file_path.as_path().ok_or("Invalid file path")?;
            let data = fs::read_to_string(path).map_err(|e| {
                error!("Failed to read config import: {}", e);
                format!("Failed to read file: {}", e)
            })?;

            let imported: ConfigExport = serde_json::from_str(&data).map_err(|e| {
                error!("Failed to parse config import: {}", e);
                format!("Invalid config file: {}", e)
            })?;

            // Update settings
            {
                let mut current = state.settings.lock().map_err(|e| e.to_string())?;
                *current = imported.settings.clone();
            }
            config::save_settings(&imported.settings)?;

            // Update mappings
            {
                let mut current = state.mappings.lock().map_err(|e| e.to_string())?;
                *current = imported.mappings.clone();
            }
            config::save_mappings(&imported.mappings)?;

            info!("Config imported from {:?}", path);
            Ok(true)
        }
        None => {
            // User cancelled the dialog
            Ok(false)
        }
    }
}
