use crate::models::MidiPort;
use midir::{MidiInput, MidiOutput};

#[tauri::command]
pub fn list_midi_inputs() -> Result<Vec<MidiPort>, String> {
    let midi_in = MidiInput::new("Conduit").map_err(|e| format!("MIDI input error: {}", e))?;
    let ports = midi_in.ports();
    let mut result = Vec::new();
    for (i, port) in ports.iter().enumerate() {
        if let Ok(name) = midi_in.port_name(port) {
            result.push(MidiPort { name, index: i });
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn list_midi_outputs() -> Result<Vec<MidiPort>, String> {
    let midi_out = MidiOutput::new("Conduit").map_err(|e| format!("MIDI output error: {}", e))?;
    let ports = midi_out.ports();
    let mut result = Vec::new();
    for (i, port) in ports.iter().enumerate() {
        if let Ok(name) = midi_out.port_name(port) {
            result.push(MidiPort { name, index: i });
        }
    }
    Ok(result)
}
