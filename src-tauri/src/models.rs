use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub osc_listen_port: u16,
    pub osc_listen_protocol: OscListenProtocol,
    pub osc_send_host: String,
    pub osc_send_port: u16,
    pub osc_send_protocol: OscSendProtocol,
    pub osc_tcp_send_timeout_ms: u64,
    pub midi_input_port_name: Option<String>,
    pub midi_output_port_name: Option<String>,
    pub engine_auto_start: bool,
    #[serde(default)]
    pub launch_on_startup: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            osc_listen_port: 8000,
            osc_listen_protocol: OscListenProtocol::Udp,
            osc_send_host: "127.0.0.1".to_string(),
            osc_send_port: 9000,
            osc_send_protocol: OscSendProtocol::Udp,
            osc_tcp_send_timeout_ms: 3000,
            midi_input_port_name: None,
            midi_output_port_name: None,
            engine_auto_start: false,
            launch_on_startup: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OscListenProtocol {
    Udp,
    Tcp,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OscSendProtocol {
    Udp,
    Tcp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mapping {
    pub id: String,
    pub enabled: bool,
    pub direction: Direction,
    pub osc_address: String,
    pub osc_arg_types: Vec<OscArgType>,
    pub midi_message_type: MidiMessageType,
    pub midi_channel: u8,
    pub midi_note_or_cc: u8,
    pub midi_velocity_or_value: ValueSource,
    #[serde(default)]
    pub midi_input_velocity: Option<u8>,
    pub osc_args: Vec<OscArgDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    OscToMidi,
    MidiToOsc,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MidiMessageType {
    NoteOn,
    NoteOff,
    Cc,
    ProgramChange,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValueSource {
    Static { value: u8 },
    OscArg { index: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OscArgType {
    Int,
    Float,
    String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OscArgDef {
    #[serde(rename = "type")]
    pub arg_type: OscArgType,
    pub source: OscArgSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OscArgSource {
    Static { value: serde_json::Value },
    MidiValue,
    MidiNote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiPort {
    pub name: String,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    pub running: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingActivity {
    pub timestamp: String,
    pub input_protocol: String,
    pub input_display: String,
    pub output_protocol: String,
    pub output_display: String,
    pub mapping_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnmatchedMessage {
    pub timestamp: String,
    pub protocol: String,
    pub display: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let s = Settings::default();
        assert_eq!(s.osc_listen_port, 8000);
        assert_eq!(s.osc_send_port, 9000);
        assert_eq!(s.osc_send_host, "127.0.0.1");
        assert!(!s.engine_auto_start);
        assert!(!s.launch_on_startup);
        assert!(s.midi_input_port_name.is_none());
    }

    #[test]
    fn test_settings_backward_compat_missing_launch_on_startup() {
        // Simulate a settings.json from before launch_on_startup was added
        let json = r#"{
            "osc_listen_port": 8000,
            "osc_listen_protocol": "udp",
            "osc_send_host": "127.0.0.1",
            "osc_send_port": 9000,
            "osc_send_protocol": "udp",
            "osc_tcp_send_timeout_ms": 3000,
            "midi_input_port_name": null,
            "midi_output_port_name": null,
            "engine_auto_start": false
        }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert!(!s.launch_on_startup);
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let s = Settings::default();
        let json = serde_json::to_string_pretty(&s).unwrap();
        let s2: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, s2);
    }

    #[test]
    fn test_settings_json_format() {
        let s = Settings::default();
        let v: serde_json::Value = serde_json::to_value(&s).unwrap();
        assert_eq!(v["osc_listen_protocol"], "udp");
        assert_eq!(v["osc_send_protocol"], "udp");
    }

    #[test]
    fn test_mapping_serialization_roundtrip() {
        let m = Mapping {
            id: "test-id".to_string(),
            enabled: true,
            direction: Direction::OscToMidi,
            osc_address: "/cue/go".to_string(),
            osc_arg_types: vec![],
            midi_message_type: MidiMessageType::NoteOn,
            midi_channel: 1,
            midi_note_or_cc: 60,
            midi_velocity_or_value: ValueSource::Static { value: 127 },
            midi_input_velocity: None,
            osc_args: vec![],
        };
        let json = serde_json::to_string_pretty(&m).unwrap();
        let m2: Mapping = serde_json::from_str(&json).unwrap();
        assert_eq!(m, m2);
    }

    #[test]
    fn test_mapping_json_tags() {
        let m = Mapping {
            id: "a1b2c3d4".to_string(),
            enabled: true,
            direction: Direction::OscToMidi,
            osc_address: "/cue/go".to_string(),
            osc_arg_types: vec![],
            midi_message_type: MidiMessageType::NoteOn,
            midi_channel: 1,
            midi_note_or_cc: 60,
            midi_velocity_or_value: ValueSource::Static { value: 127 },
            midi_input_velocity: None,
            osc_args: vec![],
        };
        let v: serde_json::Value = serde_json::to_value(&m).unwrap();
        assert_eq!(v["direction"], "osc_to_midi");
        assert_eq!(v["midi_message_type"], "note_on");
        assert_eq!(v["midi_velocity_or_value"]["type"], "static");
        assert_eq!(v["midi_velocity_or_value"]["value"], 127);
    }

    #[test]
    fn test_value_source_osc_arg() {
        let vs = ValueSource::OscArg { index: 0 };
        let v: serde_json::Value = serde_json::to_value(&vs).unwrap();
        assert_eq!(v["type"], "osc_arg");
        assert_eq!(v["index"], 0);
    }

    #[test]
    fn test_osc_arg_def_serialization() {
        let arg = OscArgDef {
            arg_type: OscArgType::Float,
            source: OscArgSource::MidiValue,
        };
        let v: serde_json::Value = serde_json::to_value(&arg).unwrap();
        assert_eq!(v["type"], "float");
        assert_eq!(v["source"]["type"], "midi_value");
    }

    #[test]
    fn test_midi_to_osc_mapping() {
        let m = Mapping {
            id: "m2o-1".to_string(),
            enabled: true,
            direction: Direction::MidiToOsc,
            osc_address: "/mix/volume".to_string(),
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
        };
        let json = serde_json::to_string_pretty(&m).unwrap();
        let m2: Mapping = serde_json::from_str(&json).unwrap();
        assert_eq!(m, m2);
    }

    #[test]
    fn test_mapping_backward_compat_missing_midi_input_velocity() {
        // Simulate a mappings.json from before midi_input_velocity was added
        let json = r#"{
            "id": "test-1",
            "enabled": true,
            "direction": "osc_to_midi",
            "osc_address": "/cue/go",
            "osc_arg_types": [],
            "midi_message_type": "note_on",
            "midi_channel": 1,
            "midi_note_or_cc": 60,
            "midi_velocity_or_value": {"type": "static", "value": 127},
            "osc_args": []
        }"#;
        let m: Mapping = serde_json::from_str(json).unwrap();
        assert_eq!(m.midi_input_velocity, None);
    }
}
