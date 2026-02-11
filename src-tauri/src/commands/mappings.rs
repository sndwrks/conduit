use crate::config;
use crate::models::Mapping;
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn get_mappings(state: State<AppState>) -> Result<Vec<Mapping>, String> {
    let mappings = state.mappings.lock().map_err(|e| e.to_string())?;
    Ok(mappings.clone())
}

#[tauri::command]
pub fn add_mapping(mapping: Mapping, state: State<AppState>) -> Result<String, String> {
    let mut mappings = state.mappings.lock().map_err(|e| e.to_string())?;
    let mut new_mapping = mapping;
    let id = Uuid::new_v4().to_string();
    new_mapping.id = id.clone();
    mappings.push(new_mapping);
    config::save_mappings(&mappings)?;
    Ok(id)
}

#[tauri::command]
pub fn update_mapping(mapping: Mapping, state: State<AppState>) -> Result<(), String> {
    let mut mappings = state.mappings.lock().map_err(|e| e.to_string())?;
    let idx = mappings
        .iter()
        .position(|m| m.id == mapping.id)
        .ok_or_else(|| format!("Mapping not found: {}", mapping.id))?;
    mappings[idx] = mapping;
    config::save_mappings(&mappings)
}

#[tauri::command]
pub fn delete_mapping(id: String, state: State<AppState>) -> Result<(), String> {
    let mut mappings = state.mappings.lock().map_err(|e| e.to_string())?;
    let len_before = mappings.len();
    mappings.retain(|m| m.id != id);
    if mappings.len() == len_before {
        return Err(format!("Mapping not found: {}", id));
    }
    config::save_mappings(&mappings)
}

#[tauri::command]
pub fn reorder_mappings(ids: Vec<String>, state: State<AppState>) -> Result<(), String> {
    let mut mappings = state.mappings.lock().map_err(|e| e.to_string())?;

    if ids.len() != mappings.len() {
        return Err(format!(
            "Reorder ID count ({}) does not match mapping count ({})",
            ids.len(),
            mappings.len()
        ));
    }

    let mut provided_ids: Vec<&String> = ids.iter().collect();
    let mut existing_ids: Vec<&String> = mappings.iter().map(|m| &m.id).collect();
    provided_ids.sort();
    existing_ids.sort();
    if provided_ids != existing_ids {
        return Err("Reorder IDs do not match existing mapping IDs".to_string());
    }

    let mut reordered = Vec::with_capacity(ids.len());
    for id in &ids {
        let mapping = mappings
            .iter()
            .find(|m| &m.id == id)
            .ok_or_else(|| format!("Mapping not found: {}", id))?
            .clone();
        reordered.push(mapping);
    }
    *mappings = reordered;
    config::save_mappings(&mappings)
}
