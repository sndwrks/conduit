use crate::models::MidiMessageType;
use crate::router::IncomingMessage;
use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct ParsedMidi {
    pub message_type: MidiMessageType,
    pub channel: u8,
    pub note_or_cc: u8,
    pub value: u8,
}

/// Parse a MIDI message from raw bytes.
/// Handles running status: if the first byte < 0x80, use last_status.
pub fn parse_midi_message(bytes: &[u8], last_status: &mut Option<u8>) -> Option<ParsedMidi> {
    if bytes.is_empty() {
        return None;
    }

    let (status, data_start) = if bytes[0] >= 0x80 {
        *last_status = Some(bytes[0]);
        (bytes[0], 1)
    } else {
        // Running status
        match *last_status {
            Some(s) => (s, 0),
            None => return None,
        }
    };

    let msg_type = status & 0xF0;
    let channel = (status & 0x0F) + 1; // 1-indexed

    match msg_type {
        0x90 => {
            // Note On
            let note = *bytes.get(data_start)?;
            let velocity = *bytes.get(data_start + 1)?;
            if velocity == 0 {
                // Note On with velocity 0 = Note Off
                Some(ParsedMidi {
                    message_type: MidiMessageType::NoteOff,
                    channel,
                    note_or_cc: note,
                    value: 0,
                })
            } else {
                Some(ParsedMidi {
                    message_type: MidiMessageType::NoteOn,
                    channel,
                    note_or_cc: note,
                    value: velocity,
                })
            }
        }
        0x80 => {
            // Note Off
            let note = *bytes.get(data_start)?;
            let velocity = *bytes.get(data_start + 1)?;
            Some(ParsedMidi {
                message_type: MidiMessageType::NoteOff,
                channel,
                note_or_cc: note,
                value: velocity,
            })
        }
        0xB0 => {
            // CC
            let cc = *bytes.get(data_start)?;
            let value = *bytes.get(data_start + 1)?;
            Some(ParsedMidi {
                message_type: MidiMessageType::Cc,
                channel,
                note_or_cc: cc,
                value,
            })
        }
        0xC0 => {
            // Program Change (2-byte message: status + program number)
            let program = *bytes.get(data_start)?;
            Some(ParsedMidi {
                message_type: MidiMessageType::ProgramChange,
                channel,
                note_or_cc: program,
                value: 0,
            })
        }
        _ => {
            // Unsupported message type
            None
        }
    }
}

pub fn open_input(
    port_name: &str,
    tx: mpsc::UnboundedSender<IncomingMessage>,
) -> Result<MidiInputConnection<()>, String> {
    let midi_in =
        MidiInput::new("Conduit").map_err(|e| format!("Failed to create MIDI input: {}", e))?;
    let ports = midi_in.ports();
    let port = ports
        .iter()
        .find(|p| midi_in.port_name(p).unwrap_or_default() == port_name)
        .ok_or_else(|| format!("MIDI input port not found: {}", port_name))?
        .clone();

    let mut last_status: Option<u8> = None;
    midi_in
        .connect(
            &port,
            "conduit-in",
            move |_timestamp, bytes, _| {
                if let Some(parsed) = parse_midi_message(bytes, &mut last_status) {
                    let msg = IncomingMessage::Midi {
                        message_type: parsed.message_type,
                        channel: parsed.channel,
                        note_or_cc: parsed.note_or_cc,
                        value: parsed.value,
                    };
                    let _ = tx.send(msg);
                }
            },
            (),
        )
        .map_err(|e| format!("Failed to connect MIDI input: {}", e))
}

pub fn open_output(
    port_name: &str,
) -> Result<Arc<Mutex<MidiOutputConnection>>, String> {
    let midi_out =
        MidiOutput::new("Conduit").map_err(|e| format!("Failed to create MIDI output: {}", e))?;
    let ports = midi_out.ports();
    let port = ports
        .iter()
        .find(|p| midi_out.port_name(p).unwrap_or_default() == port_name)
        .ok_or_else(|| format!("MIDI output port not found: {}", port_name))?
        .clone();

    let conn = midi_out
        .connect(&port, "conduit-out")
        .map_err(|e| format!("Failed to connect MIDI output: {}", e))?;
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn enumerate_ports_hash() -> (Vec<String>, Vec<String>) {
    let inputs = MidiInput::new("Conduit-enum")
        .map(|m| {
            m.ports()
                .iter()
                .filter_map(|p| m.port_name(p).ok())
                .collect()
        })
        .unwrap_or_default();

    let outputs = MidiOutput::new("Conduit-enum")
        .map(|m| {
            m.ports()
                .iter()
                .filter_map(|p| m.port_name(p).ok())
                .collect()
        })
        .unwrap_or_default();

    (inputs, outputs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_note_on() {
        let mut last = None;
        let result = parse_midi_message(&[0x90, 60, 100], &mut last);
        let p = result.unwrap();
        assert!(matches!(p.message_type, MidiMessageType::NoteOn));
        assert_eq!(p.channel, 1);
        assert_eq!(p.note_or_cc, 60);
        assert_eq!(p.value, 100);
    }

    #[test]
    fn test_parse_note_on_velocity_zero_is_note_off() {
        let mut last = None;
        let result = parse_midi_message(&[0x90, 60, 0], &mut last);
        let p = result.unwrap();
        assert!(matches!(p.message_type, MidiMessageType::NoteOff));
        assert_eq!(p.value, 0);
    }

    #[test]
    fn test_parse_note_off() {
        let mut last = None;
        let result = parse_midi_message(&[0x80, 60, 64], &mut last);
        let p = result.unwrap();
        assert!(matches!(p.message_type, MidiMessageType::NoteOff));
        assert_eq!(p.channel, 1);
        assert_eq!(p.note_or_cc, 60);
        assert_eq!(p.value, 64);
    }

    #[test]
    fn test_parse_cc() {
        let mut last = None;
        let result = parse_midi_message(&[0xB0, 7, 100], &mut last);
        let p = result.unwrap();
        assert!(matches!(p.message_type, MidiMessageType::Cc));
        assert_eq!(p.note_or_cc, 7);
        assert_eq!(p.value, 100);
    }

    #[test]
    fn test_parse_channel_extraction() {
        let mut last = None;
        // Channel 10 (0-indexed 9)
        let result = parse_midi_message(&[0x99, 60, 100], &mut last);
        let p = result.unwrap();
        assert_eq!(p.channel, 10);
    }

    #[test]
    fn test_parse_running_status() {
        let mut last = None;
        // First: full message
        parse_midi_message(&[0x90, 60, 100], &mut last);
        assert_eq!(last, Some(0x90));
        // Second: running status (no status byte)
        let result = parse_midi_message(&[62, 80], &mut last);
        let p = result.unwrap();
        assert!(matches!(p.message_type, MidiMessageType::NoteOn));
        assert_eq!(p.note_or_cc, 62);
        assert_eq!(p.value, 80);
    }

    #[test]
    fn test_parse_running_status_no_previous() {
        let mut last = None;
        let result = parse_midi_message(&[60, 100], &mut last);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_program_change() {
        let mut last = None;
        let result = parse_midi_message(&[0xC0, 5], &mut last);
        let p = result.unwrap();
        assert!(matches!(p.message_type, MidiMessageType::ProgramChange));
        assert_eq!(p.channel, 1);
        assert_eq!(p.note_or_cc, 5);
        assert_eq!(p.value, 0);
    }

    #[test]
    fn test_parse_unsupported_type() {
        let mut last = None;
        // Pitch bend - still unsupported
        let result = parse_midi_message(&[0xE0, 0, 64], &mut last);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_empty() {
        let mut last = None;
        let result = parse_midi_message(&[], &mut last);
        assert!(result.is_none());
    }
}
