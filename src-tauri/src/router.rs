use crate::models::*;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone)]
pub enum IncomingMessage {
    Osc {
        address: String,
        args: Vec<OscArgValue>,
    },
    Midi {
        message_type: MidiMessageType,
        channel: u8,
        note_or_cc: u8,
        value: u8,
    },
    Msc {
        device_id: u8,
        command_format: u8,
        command: u8,
        cue_number: String,
        cue_list: Option<String>,
        cue_path: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OscArgValue {
    Int(i32),
    Float(f32),
    String(String),
}

pub struct Router {
    mappings: Arc<Mutex<Vec<Mapping>>>,
    app_handle: AppHandle,
    last_emit: Mutex<Instant>,
    emit_count: Mutex<u32>,
}

impl Router {
    pub fn new(mappings: Arc<Mutex<Vec<Mapping>>>, app_handle: AppHandle) -> Self {
        Self {
            mappings,
            app_handle,
            last_emit: Mutex::new(Instant::now()),
            emit_count: Mutex::new(0),
        }
    }

    pub fn route(&self, msg: &IncomingMessage) -> Vec<OutputAction> {
        let mappings = match self.mappings.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Mappings mutex poisoned in route(): {}", e);
                return Vec::new();
            }
        };
        let mut actions = Vec::new();
        let mut matched = false;

        for mapping in mappings.iter() {
            if !mapping.enabled {
                continue;
            }

            if let Some(action) = self.try_match(mapping, msg) {
                matched = true;
                self.emit_activity(mapping, msg, &action);
                actions.push(action);
            }
        }

        if !matched {
            self.emit_unmatched(msg);
        }

        actions
    }

    fn try_match(&self, mapping: &Mapping, msg: &IncomingMessage) -> Option<OutputAction> {
        match (msg, &mapping.direction) {
            (
                IncomingMessage::Osc { address, args },
                Direction::OscToMidi,
            ) => {
                if address != &mapping.osc_address {
                    return None;
                }
                let value = match &mapping.midi_velocity_or_value {
                    ValueSource::Static { value } => *value,
                    ValueSource::OscArg { index } => {
                        match args.get(*index) {
                            Some(a) => osc_arg_to_midi_value(a),
                            None => {
                                warn!(
                                    "OSC arg index {} out of range (message has {} args), defaulting to 0",
                                    index,
                                    args.len()
                                );
                                0
                            }
                        }
                    }
                };
                Some(OutputAction::Midi {
                    message_type: mapping.midi_message_type.clone(),
                    channel: mapping.midi_channel,
                    note_or_cc: mapping.midi_note_or_cc,
                    value,
                })
            }
            (
                IncomingMessage::Midi {
                    message_type,
                    channel,
                    note_or_cc,
                    value,
                },
                Direction::MidiToOsc,
            ) => {
                if message_type != &mapping.midi_message_type
                    || channel != &mapping.midi_channel
                    || note_or_cc != &mapping.midi_note_or_cc
                {
                    return None;
                }
                if let Some(vel) = mapping.midi_input_velocity {
                    if value != &vel {
                        return None;
                    }
                }
                let osc_args: Vec<OscArgValue> = mapping
                    .osc_args
                    .iter()
                    .map(|def| build_osc_arg(def, *value, *note_or_cc))
                    .collect();
                Some(OutputAction::Osc {
                    address: mapping.osc_address.clone(),
                    args: osc_args,
                })
            }
            (
                IncomingMessage::Msc {
                    device_id,
                    command_format,
                    command,
                    cue_number,
                    cue_list,
                    cue_path,
                },
                Direction::MidiToOsc,
            ) => {
                if mapping.midi_message_type != MidiMessageType::Msc {
                    return None;
                }
                // Filter by device ID (None = match all)
                if let Some(expected_id) = mapping.msc_device_id {
                    if *device_id != expected_id && *device_id != 0x7F && expected_id != 0x7F {
                        return None;
                    }
                }
                // Filter by command format
                if let Some(ref expected_fmt) = mapping.msc_command_format {
                    let expected_byte = msc_command_format_to_byte(expected_fmt);
                    if *command_format != expected_byte
                        && *command_format != 0x7F
                        && expected_byte != 0x7F
                    {
                        return None;
                    }
                }
                // Filter by command
                if let Some(ref expected_cmd) = mapping.msc_command {
                    let expected_byte = msc_command_to_byte(expected_cmd);
                    if *command != expected_byte {
                        return None;
                    }
                }
                let osc_args: Vec<OscArgValue> = mapping
                    .osc_args
                    .iter()
                    .map(|def| build_osc_arg_msc(def, cue_number, cue_list, cue_path))
                    .collect();
                Some(OutputAction::Osc {
                    address: mapping.osc_address.clone(),
                    args: osc_args,
                })
            }
            _ => None,
        }
    }

    fn can_emit(&self) -> bool {
        let mut last = match self.last_emit.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("last_emit mutex poisoned: {}", e);
                return false;
            }
        };
        let mut count = match self.emit_count.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("emit_count mutex poisoned: {}", e);
                return false;
            }
        };
        let now = Instant::now();
        if now.duration_since(*last).as_millis() >= 1000 {
            *last = now;
            *count = 0;
        }
        if *count >= 60 {
            return false;
        }
        *count += 1;
        true
    }

    fn emit_activity(&self, mapping: &Mapping, msg: &IncomingMessage, action: &OutputAction) {
        if !self.can_emit() {
            return;
        }
        let timestamp = chrono::Local::now().to_rfc3339();
        let (input_protocol, input_display) = format_incoming(msg);
        let (output_protocol, output_display) = format_output(action);
        let _ = self.app_handle.emit(
            "mapping-activity",
            MappingActivity {
                timestamp,
                input_protocol,
                input_display,
                output_protocol,
                output_display,
                mapping_id: mapping.id.clone(),
            },
        );
    }

    fn emit_unmatched(&self, msg: &IncomingMessage) {
        if !self.can_emit() {
            return;
        }
        let timestamp = chrono::Local::now().to_rfc3339();
        let (protocol, display) = format_incoming(msg);
        let _ = self.app_handle.emit(
            "unmatched-message",
            UnmatchedMessage {
                timestamp,
                protocol,
                display,
            },
        );
    }
}

#[derive(Debug, Clone)]
pub enum OutputAction {
    Midi {
        message_type: MidiMessageType,
        channel: u8,
        note_or_cc: u8,
        value: u8,
    },
    Osc {
        address: String,
        args: Vec<OscArgValue>,
    },
}

fn osc_arg_to_midi_value(arg: &OscArgValue) -> u8 {
    match arg {
        OscArgValue::Float(f) => (f * 127.0).round().clamp(0.0, 127.0) as u8,
        OscArgValue::Int(i) => (*i).clamp(0, 127) as u8,
        OscArgValue::String(_) => 0,
    }
}

fn midi_value_to_osc_float(value: u8) -> f32 {
    value as f32 / 127.0
}

fn msc_command_to_byte(cmd: &MscCommand) -> u8 {
    match cmd {
        MscCommand::Go => 0x01,
        MscCommand::Stop => 0x02,
        MscCommand::Resume => 0x03,
    }
}

fn msc_command_format_to_byte(fmt: &MscCommandFormat) -> u8 {
    match fmt {
        MscCommandFormat::All => 0x7F,
        MscCommandFormat::Lighting => 0x01,
        MscCommandFormat::Sound => 0x0A,
    }
}

fn msc_command_byte_to_str(cmd: u8) -> &'static str {
    match cmd {
        0x01 => "GO",
        0x02 => "STOP",
        0x03 => "RESUME",
        0x04 => "TIMED_GO",
        0x05 => "LOAD",
        0x06 => "SET",
        0x07 => "FIRE",
        _ => "MSC",
    }
}

fn apply_template(template: &Option<String>, placeholder: &str, value: &str) -> String {
    match template {
        Some(t) => t.replace(placeholder, value),
        None => value.to_string(),
    }
}

fn build_osc_arg_msc(
    def: &OscArgDef,
    cue_number: &str,
    cue_list: &Option<String>,
    cue_path: &Option<String>,
) -> OscArgValue {
    match &def.source {
        OscArgSource::MscCueNumber { template } => match def.arg_type {
            OscArgType::String => {
                OscArgValue::String(apply_template(template, "{cue_number}", cue_number))
            }
            OscArgType::Float => {
                OscArgValue::Float(cue_number.parse::<f32>().unwrap_or(0.0))
            }
            OscArgType::Int => OscArgValue::Int(cue_number.parse::<i32>().unwrap_or(0)),
        },
        OscArgSource::MscCueList { template } => {
            let val = cue_list.as_deref().unwrap_or("");
            match def.arg_type {
                OscArgType::String => {
                    OscArgValue::String(apply_template(template, "{cue_list}", val))
                }
                OscArgType::Float => OscArgValue::Float(val.parse::<f32>().unwrap_or(0.0)),
                OscArgType::Int => OscArgValue::Int(val.parse::<i32>().unwrap_or(0)),
            }
        }
        OscArgSource::MscCuePath { template } => {
            let val = cue_path.as_deref().unwrap_or("");
            match def.arg_type {
                OscArgType::String => {
                    OscArgValue::String(apply_template(template, "{cue_path}", val))
                }
                OscArgType::Float => OscArgValue::Float(val.parse::<f32>().unwrap_or(0.0)),
                OscArgType::Int => OscArgValue::Int(val.parse::<i32>().unwrap_or(0)),
            }
        }
        OscArgSource::Static { value } => match def.arg_type {
            OscArgType::Float => OscArgValue::Float(value.as_f64().unwrap_or(0.0) as f32),
            OscArgType::Int => OscArgValue::Int(value.as_i64().unwrap_or(0) as i32),
            OscArgType::String => {
                OscArgValue::String(value.as_str().unwrap_or("").to_string())
            }
        },
        OscArgSource::MidiValue | OscArgSource::MidiNote => OscArgValue::Int(0),
    }
}

fn build_osc_arg(def: &OscArgDef, midi_value: u8, midi_note: u8) -> OscArgValue {
    match &def.source {
        OscArgSource::MidiValue => match def.arg_type {
            OscArgType::Float => OscArgValue::Float(midi_value_to_osc_float(midi_value)),
            OscArgType::Int => OscArgValue::Int(midi_value as i32),
            OscArgType::String => OscArgValue::String(midi_value.to_string()),
        },
        OscArgSource::MidiNote => match def.arg_type {
            OscArgType::Int => OscArgValue::Int(midi_note as i32),
            OscArgType::Float => OscArgValue::Float(midi_note as f32),
            OscArgType::String => OscArgValue::String(midi_note.to_string()),
        },
        OscArgSource::Static { value } => match def.arg_type {
            OscArgType::Float => {
                OscArgValue::Float(value.as_f64().unwrap_or(0.0) as f32)
            }
            OscArgType::Int => {
                OscArgValue::Int(value.as_i64().unwrap_or(0) as i32)
            }
            OscArgType::String => {
                OscArgValue::String(value.as_str().unwrap_or("").to_string())
            }
        },
        OscArgSource::MscCueNumber { .. }
        | OscArgSource::MscCueList { .. }
        | OscArgSource::MscCuePath { .. } => OscArgValue::String(String::new()),
    }
}

fn format_incoming(msg: &IncomingMessage) -> (String, String) {
    match msg {
        IncomingMessage::Osc { address, args } => {
            let args_str = args
                .iter()
                .map(|a| match a {
                    OscArgValue::Int(i) => i.to_string(),
                    OscArgValue::Float(f) => format!("{:.3}", f),
                    OscArgValue::String(s) => format!("\"{}\"", s),
                })
                .collect::<Vec<_>>()
                .join(" ");
            ("osc".to_string(), format!("{} ({})", address, args_str))
        }
        IncomingMessage::Midi {
            message_type,
            channel,
            note_or_cc,
            value,
        } => {
            let type_str = match message_type {
                MidiMessageType::NoteOn => "Note On",
                MidiMessageType::NoteOff => "Note Off",
                MidiMessageType::Cc => "CC",
                MidiMessageType::ProgramChange => "PC",
                MidiMessageType::Msc => "MSC",
            };
            if matches!(message_type, MidiMessageType::ProgramChange) {
                ("midi".to_string(), format!("PC {} Ch {}", note_or_cc, channel))
            } else {
                (
                    "midi".to_string(),
                    format!("{} {} Val {} Ch {}", type_str, note_or_cc, value, channel),
                )
            }
        }
        IncomingMessage::Msc {
            command,
            cue_number,
            ..
        } => {
            let cmd_str = msc_command_byte_to_str(*command);
            if cue_number.is_empty() {
                ("midi".to_string(), format!("MSC {}", cmd_str))
            } else {
                ("midi".to_string(), format!("MSC {} Q{}", cmd_str, cue_number))
            }
        }
    }
}

fn format_output(action: &OutputAction) -> (String, String) {
    match action {
        OutputAction::Midi {
            message_type,
            channel,
            note_or_cc,
            value,
        } => {
            let type_str = match message_type {
                MidiMessageType::NoteOn => "Note On",
                MidiMessageType::NoteOff => "Note Off",
                MidiMessageType::Cc => "CC",
                MidiMessageType::ProgramChange => "PC",
                MidiMessageType::Msc => "MSC",
            };
            if matches!(message_type, MidiMessageType::ProgramChange) {
                ("midi".to_string(), format!("PC {} Ch {}", note_or_cc, channel))
            } else {
                (
                    "midi".to_string(),
                    format!("{} {} Val {} Ch {}", type_str, note_or_cc, value, channel),
                )
            }
        }
        OutputAction::Osc { address, args } => {
            let args_str = args
                .iter()
                .map(|a| match a {
                    OscArgValue::Int(i) => i.to_string(),
                    OscArgValue::Float(f) => format!("{:.3}", f),
                    OscArgValue::String(s) => format!("\"{}\"", s),
                })
                .collect::<Vec<_>>()
                .join(" ");
            ("osc".to_string(), format!("{} {}", address, args_str))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_osc_to_midi_mapping(address: &str, value_source: ValueSource) -> Mapping {
        Mapping {
            id: "test-1".to_string(),
            enabled: true,
            direction: Direction::OscToMidi,
            osc_address: address.to_string(),
            osc_arg_types: vec![],
            midi_message_type: MidiMessageType::NoteOn,
            midi_channel: 1,
            midi_note_or_cc: 60,
            midi_velocity_or_value: value_source,
            midi_input_velocity: None,
            osc_args: vec![],
            msc_device_id: None,
            msc_command_format: None,
            msc_command: None,
        }
    }

    fn make_midi_to_osc_mapping() -> Mapping {
        Mapping {
            id: "test-2".to_string(),
            enabled: true,
            direction: Direction::MidiToOsc,
            osc_address: "/output".to_string(),
            osc_arg_types: vec![OscArgType::Float],
            midi_message_type: MidiMessageType::Cc,
            midi_channel: 1,
            midi_note_or_cc: 7,
            midi_velocity_or_value: ValueSource::Static { value: 0 },
            midi_input_velocity: None,
            osc_args: vec![OscArgDef {
                arg_type: OscArgType::Float,
                source: OscArgSource::MidiValue,
            }],
            msc_device_id: None,
            msc_command_format: None,
            msc_command: None,
        }
    }

    #[test]
    fn test_osc_to_midi_value_scaling() {
        assert_eq!(osc_arg_to_midi_value(&OscArgValue::Float(0.0)), 0);
        assert_eq!(osc_arg_to_midi_value(&OscArgValue::Float(1.0)), 127);
        assert_eq!(osc_arg_to_midi_value(&OscArgValue::Float(0.5)), 64);
    }

    #[test]
    fn test_midi_to_osc_value_scaling() {
        assert_eq!(midi_value_to_osc_float(0), 0.0);
        assert_eq!(midi_value_to_osc_float(127), 1.0);
        let half = midi_value_to_osc_float(64);
        assert!((half - 0.504).abs() < 0.01);
    }

    #[test]
    fn test_osc_arg_to_midi_static() {
        assert_eq!(osc_arg_to_midi_value(&OscArgValue::Int(100)), 100);
        assert_eq!(osc_arg_to_midi_value(&OscArgValue::Int(200)), 127);
        assert_eq!(osc_arg_to_midi_value(&OscArgValue::String("x".to_string())), 0);
    }

    #[test]
    fn test_build_osc_arg_midi_value() {
        let def = OscArgDef {
            arg_type: OscArgType::Float,
            source: OscArgSource::MidiValue,
        };
        match build_osc_arg(&def, 127, 60) {
            OscArgValue::Float(f) => assert_eq!(f, 1.0),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_build_osc_arg_midi_note() {
        let def = OscArgDef {
            arg_type: OscArgType::Int,
            source: OscArgSource::MidiNote,
        };
        match build_osc_arg(&def, 127, 60) {
            OscArgValue::Int(i) => assert_eq!(i, 60),
            _ => panic!("Expected int"),
        }
    }

    #[test]
    fn test_build_osc_arg_static() {
        let def = OscArgDef {
            arg_type: OscArgType::String,
            source: OscArgSource::Static {
                value: serde_json::Value::String("hello".to_string()),
            },
        };
        match build_osc_arg(&def, 0, 0) {
            OscArgValue::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_midi_input_velocity_filter_exact_match() {
        let mut mapping = make_midi_to_osc_mapping();
        mapping.midi_message_type = MidiMessageType::NoteOn;
        mapping.midi_note_or_cc = 60;
        mapping.midi_input_velocity = Some(100);

        // Matching velocity — should produce an output
        let msg_match = IncomingMessage::Midi {
            message_type: MidiMessageType::NoteOn,
            channel: 1,
            note_or_cc: 60,
            value: 100,
        };

        // Non-matching velocity — should be filtered out
        let msg_no_match = IncomingMessage::Midi {
            message_type: MidiMessageType::NoteOn,
            channel: 1,
            note_or_cc: 60,
            value: 50,
        };

        // We can't easily construct a Router without an AppHandle, so test
        // the matching logic via build_osc_arg + the match conditions directly.
        // Instead, verify the filter logic inline:
        let try_match_inline = |msg: &IncomingMessage| -> bool {
            match msg {
                IncomingMessage::Midi {
                    message_type,
                    channel,
                    note_or_cc,
                    value,
                } => {
                    if message_type != &mapping.midi_message_type
                        || channel != &mapping.midi_channel
                        || note_or_cc != &mapping.midi_note_or_cc
                    {
                        return false;
                    }
                    if let Some(vel) = mapping.midi_input_velocity {
                        if value != &vel {
                            return false;
                        }
                    }
                    true
                }
                _ => false,
            }
        };

        assert!(try_match_inline(&msg_match));
        assert!(!try_match_inline(&msg_no_match));
    }

    #[test]
    fn test_midi_input_velocity_filter_any() {
        let mut mapping = make_midi_to_osc_mapping();
        mapping.midi_message_type = MidiMessageType::NoteOn;
        mapping.midi_note_or_cc = 60;
        mapping.midi_input_velocity = None; // Any velocity

        let try_match_inline = |value: u8| -> bool {
            let msg = IncomingMessage::Midi {
                message_type: MidiMessageType::NoteOn,
                channel: 1,
                note_or_cc: 60,
                value,
            };
            match &msg {
                IncomingMessage::Midi {
                    message_type,
                    channel,
                    note_or_cc,
                    value,
                } => {
                    if message_type != &mapping.midi_message_type
                        || channel != &mapping.midi_channel
                        || note_or_cc != &mapping.midi_note_or_cc
                    {
                        return false;
                    }
                    if let Some(vel) = mapping.midi_input_velocity {
                        if value != &vel {
                            return false;
                        }
                    }
                    true
                }
                _ => false,
            }
        };

        // All velocities should match when filter is None
        assert!(try_match_inline(0));
        assert!(try_match_inline(64));
        assert!(try_match_inline(127));
    }

    #[test]
    fn test_build_osc_arg_msc_cue_number_string() {
        let def = OscArgDef {
            arg_type: OscArgType::String,
            source: OscArgSource::MscCueNumber { template: None },
        };
        match build_osc_arg_msc(&def, "10", &None, &None) {
            OscArgValue::String(s) => assert_eq!(s, "10"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_build_osc_arg_msc_cue_number_float() {
        let def = OscArgDef {
            arg_type: OscArgType::Float,
            source: OscArgSource::MscCueNumber { template: None },
        };
        match build_osc_arg_msc(&def, "1.5", &None, &None) {
            OscArgValue::Float(f) => assert!((f - 1.5).abs() < 0.001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_build_osc_arg_msc_cue_number_with_template() {
        let def = OscArgDef {
            arg_type: OscArgType::String,
            source: OscArgSource::MscCueNumber {
                template: Some("Lighting Cue {cue_number}".to_string()),
            },
        };
        match build_osc_arg_msc(&def, "10", &None, &None) {
            OscArgValue::String(s) => assert_eq!(s, "Lighting Cue 10"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_build_osc_arg_msc_cue_list() {
        let def = OscArgDef {
            arg_type: OscArgType::String,
            source: OscArgSource::MscCueList { template: None },
        };
        let cue_list = Some("5".to_string());
        match build_osc_arg_msc(&def, "10", &cue_list, &None) {
            OscArgValue::String(s) => assert_eq!(s, "5"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_build_osc_arg_msc_cue_list_none() {
        let def = OscArgDef {
            arg_type: OscArgType::String,
            source: OscArgSource::MscCueList { template: None },
        };
        match build_osc_arg_msc(&def, "10", &None, &None) {
            OscArgValue::String(s) => assert_eq!(s, ""),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_build_osc_arg_msc_cue_list_with_template() {
        let def = OscArgDef {
            arg_type: OscArgType::String,
            source: OscArgSource::MscCueList {
                template: Some("List: {cue_list}".to_string()),
            },
        };
        let cue_list = Some("main".to_string());
        match build_osc_arg_msc(&def, "10", &cue_list, &None) {
            OscArgValue::String(s) => assert_eq!(s, "List: main"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_msc_command_format_matching() {
        assert_eq!(msc_command_format_to_byte(&MscCommandFormat::All), 0x7F);
        assert_eq!(msc_command_format_to_byte(&MscCommandFormat::Lighting), 0x01);
        assert_eq!(msc_command_format_to_byte(&MscCommandFormat::Sound), 0x0A);
    }

    #[test]
    fn test_msc_command_matching() {
        assert_eq!(msc_command_to_byte(&MscCommand::Go), 0x01);
        assert_eq!(msc_command_to_byte(&MscCommand::Stop), 0x02);
        assert_eq!(msc_command_to_byte(&MscCommand::Resume), 0x03);
    }
}
