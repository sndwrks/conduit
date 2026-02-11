use crate::models::{Mapping, Settings};
use log::{error, info};
use std::fs;
use std::path::PathBuf;

fn default_config_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home.join(".sndwrks-conduit"))
}

pub fn config_dir() -> Result<PathBuf, String> {
    let dir = default_config_dir()?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config directory: {}", e))?;
    Ok(dir)
}

pub fn load_settings() -> Result<Settings, String> {
    let path = config_dir()?.join("settings.json");
    if !path.exists() {
        info!("No settings.json found, creating defaults");
        let defaults = Settings::default();
        save_settings(&defaults)?;
        return Ok(defaults);
    }
    let data = fs::read_to_string(&path).map_err(|e| {
        error!("Failed to read settings: {}", e);
        format!("Failed to read settings: {}", e)
    })?;
    serde_json::from_str(&data).map_err(|e| {
        error!("Failed to parse settings: {}", e);
        format!("Failed to parse settings: {}", e)
    })
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let path = config_dir()?.join("settings.json");
    let data = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, data).map_err(|e| {
        error!("Failed to write settings: {}", e);
        format!("Failed to write settings: {}", e)
    })
}

pub fn load_mappings() -> Result<Vec<Mapping>, String> {
    let path = config_dir()?.join("mappings.json");
    if !path.exists() {
        info!("No mappings.json found, creating defaults");
        let defaults: Vec<Mapping> = vec![];
        save_mappings(&defaults)?;
        return Ok(defaults);
    }
    let data = fs::read_to_string(&path).map_err(|e| {
        error!("Failed to read mappings: {}", e);
        format!("Failed to read mappings: {}", e)
    })?;
    serde_json::from_str(&data).map_err(|e| {
        error!("Failed to parse mappings: {}", e);
        format!("Failed to parse mappings: {}", e)
    })
}

pub fn save_mappings(mappings: &[Mapping]) -> Result<(), String> {
    let dir = config_dir()?;
    let path = dir.join("mappings.json");
    let tmp_path = dir.join("mappings.json.tmp");
    let data = serde_json::to_string_pretty(mappings)
        .map_err(|e| format!("Failed to serialize mappings: {}", e))?;
    fs::write(&tmp_path, &data).map_err(|e| {
        error!("Failed to write temp mappings file: {}", e);
        format!("Failed to write temp mappings file: {}", e)
    })?;
    fs::rename(&tmp_path, &path).map_err(|e| {
        error!("Failed to rename mappings file: {}", e);
        format!("Failed to rename mappings file: {}", e)
    })
}

// Functions that accept a custom dir for testing
#[cfg(test)]
fn load_settings_from(dir: &PathBuf) -> Result<Settings, String> {
    let path = dir.join("settings.json");
    if !path.exists() {
        let defaults = Settings::default();
        save_settings_to(dir, &defaults)?;
        return Ok(defaults);
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings: {}", e))?;
    serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse settings: {}", e))
}

#[cfg(test)]
fn save_settings_to(dir: &PathBuf, settings: &Settings) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {}", e))?;
    let path = dir.join("settings.json");
    let data = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, data)
        .map_err(|e| format!("Failed to write settings: {}", e))
}

#[cfg(test)]
fn load_mappings_from(dir: &PathBuf) -> Result<Vec<Mapping>, String> {
    let path = dir.join("mappings.json");
    if !path.exists() {
        let defaults: Vec<Mapping> = vec![];
        save_mappings_to(dir, &defaults)?;
        return Ok(defaults);
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read mappings: {}", e))?;
    serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse mappings: {}", e))
}

#[cfg(test)]
fn save_mappings_to(dir: &PathBuf, mappings: &[Mapping]) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {}", e))?;
    let path = dir.join("mappings.json");
    let tmp_path = dir.join("mappings.json.tmp");
    let data = serde_json::to_string_pretty(mappings)
        .map_err(|e| format!("Failed to serialize mappings: {}", e))?;
    fs::write(&tmp_path, &data)
        .map_err(|e| format!("Failed to write temp mappings file: {}", e))?;
    fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to rename mappings file: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    #[test]
    fn test_load_settings_creates_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".sndwrks-conduit");
        let settings = load_settings_from(&dir).unwrap();
        assert_eq!(settings, Settings::default());
        let path = dir.join("settings.json");
        assert!(path.exists());
    }

    #[test]
    fn test_save_and_load_settings() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".sndwrks-conduit");
        let mut settings = Settings::default();
        settings.osc_listen_port = 9999;
        save_settings_to(&dir, &settings).unwrap();
        let loaded = load_settings_from(&dir).unwrap();
        assert_eq!(loaded.osc_listen_port, 9999);
    }

    #[test]
    fn test_load_mappings_creates_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".sndwrks-conduit");
        let mappings = load_mappings_from(&dir).unwrap();
        assert!(mappings.is_empty());
        let path = dir.join("mappings.json");
        assert!(path.exists());
    }

    #[test]
    fn test_save_and_load_mappings() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".sndwrks-conduit");
        let mappings = vec![Mapping {
            id: "test-1".to_string(),
            enabled: true,
            direction: Direction::OscToMidi,
            osc_address: "/test".to_string(),
            osc_arg_types: vec![],
            midi_message_type: MidiMessageType::NoteOn,
            midi_channel: 1,
            midi_note_or_cc: 60,
            midi_velocity_or_value: ValueSource::Static { value: 127 },
            midi_input_velocity: None,
            osc_args: vec![],
        }];
        save_mappings_to(&dir, &mappings).unwrap();
        let loaded = load_mappings_from(&dir).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "test-1");
    }

    #[test]
    fn test_atomic_write_mappings() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".sndwrks-conduit");
        let mappings = vec![Mapping {
            id: "atomic-test".to_string(),
            enabled: true,
            direction: Direction::MidiToOsc,
            osc_address: "/atomic".to_string(),
            osc_arg_types: vec![],
            midi_message_type: MidiMessageType::Cc,
            midi_channel: 1,
            midi_note_or_cc: 7,
            midi_velocity_or_value: ValueSource::Static { value: 0 },
            midi_input_velocity: None,
            osc_args: vec![],
        }];
        save_mappings_to(&dir, &mappings).unwrap();
        let tmp_path = dir.join("mappings.json.tmp");
        assert!(!tmp_path.exists());
        let path = dir.join("mappings.json");
        assert!(path.exists());
    }
}
