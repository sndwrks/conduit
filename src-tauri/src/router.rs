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
                IncomingMessage::Osc { address, args },
                Direction::OscToOsc,
            ) => {
                if address != &mapping.osc_address {
                    return None;
                }
                let output_address = if mapping.osc_output_address.is_empty() {
                    mapping.osc_address.clone()
                } else {
                    mapping.osc_output_address.clone()
                };
                let output_args = match &mapping.osc_transform {
                    Some(transform) => args
                        .iter()
                        .map(|a| apply_transform(a, transform))
                        .collect(),
                    None => args.clone(),
                };
                Some(OutputAction::Osc {
                    address: output_address,
                    args: output_args,
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

fn apply_transform(arg: &OscArgValue, transform: &OscTransform) -> OscArgValue {
    match arg {
        OscArgValue::Float(f) => {
            OscArgValue::Float(transform_value(*f as f64, transform) as f32)
        }
        OscArgValue::Int(i) => {
            OscArgValue::Int(transform_value(*i as f64, transform).round() as i32)
        }
        OscArgValue::String(_) => arg.clone(),
    }
}

fn transform_value(input: f64, t: &OscTransform) -> f64 {
    // Calibrated mode uses its own point table — bypass range normalization
    if t.curve == TransformCurve::Calibrated {
        return calibrated_interpolate(input, &t.calibration_points);
    }

    let in_range = t.input_max - t.input_min;
    if in_range.abs() < f64::EPSILON {
        return t.output_min;
    }

    // Clamp input to declared range
    let clamped = input.clamp(t.input_min.min(t.input_max), t.input_min.max(t.input_max));

    // Normalize to 0.0–1.0
    let normalized = (clamped - t.input_min) / in_range;

    // Apply curve
    let curved = match t.curve {
        TransformCurve::Linear => normalized,
        TransformCurve::Logarithmic => {
            // Fader-style log curve (power curve x^3):
            // - Compressed at bottom (small input changes = tiny output changes)
            // - More resolution at top (where it matters for volume/gain)
            // This matches the behavior of a typical audio fader or log pot.
            normalized * normalized * normalized
        }
        TransformCurve::Calibrated => unreachable!(),
    };

    // Map curved 0–1 to output range
    let out_range = t.output_max - t.output_min;
    t.output_min + curved * out_range
}

fn calibrated_interpolate(input: f64, points: &[CalibrationPoint]) -> f64 {
    if points.is_empty() {
        return 0.0;
    }
    if points.len() == 1 {
        return points[0].output;
    }
    if points.len() == 2 {
        // Only two points — linear interpolation (spline needs 3+)
        let p0 = &points[0];
        let p1 = &points[1];
        let range = p1.input - p0.input;
        if range.abs() < f64::EPSILON {
            return p0.output;
        }
        let t = ((input - p0.input) / range).clamp(0.0, 1.0);
        return p0.output + t * (p1.output - p0.output);
    }

    // Clamp to endpoints
    let first = &points[0];
    let last = &points[points.len() - 1];
    if input <= first.input {
        return first.output;
    }
    if input >= last.input {
        return last.output;
    }

    // Monotone cubic Hermite interpolation (Fritsch-Carlson method).
    // Smooth curve through all points without overshoot/undershoot.
    monotone_cubic_interpolate(input, points)
}

fn monotone_cubic_interpolate(x: f64, points: &[CalibrationPoint]) -> f64 {
    let n = points.len();
    let segments = n - 1;

    // Step 1: Compute secants (slopes between adjacent points)
    let delta: Vec<f64> = (0..segments)
        .map(|i| {
            let dx = points[i + 1].input - points[i].input;
            if dx.abs() < f64::EPSILON {
                0.0
            } else {
                (points[i + 1].output - points[i].output) / dx
            }
        })
        .collect();

    // Step 2: Compute initial tangents at each point
    let mut m = vec![0.0; n];
    m[0] = delta[0];
    m[segments] = delta[segments - 1];
    for i in 1..segments {
        // Average of neighboring secants (only if same sign)
        if delta[i - 1] * delta[i] > 0.0 {
            m[i] = (delta[i - 1] + delta[i]) / 2.0;
        } else {
            m[i] = 0.0;
        }
    }

    // Step 3: Fritsch-Carlson monotonicity correction
    for i in 0..segments {
        if delta[i].abs() < f64::EPSILON {
            m[i] = 0.0;
            m[i + 1] = 0.0;
        } else {
            let alpha = m[i] / delta[i];
            let beta = m[i + 1] / delta[i];
            // Ensure we stay in the monotonicity region
            let s = alpha * alpha + beta * beta;
            if s > 9.0 {
                let tau = 3.0 / s.sqrt();
                m[i] = tau * alpha * delta[i];
                m[i + 1] = tau * beta * delta[i];
            }
        }
    }

    // Step 4: Find segment and evaluate cubic Hermite
    let mut seg = segments - 1;
    for i in 0..segments - 1 {
        if x < points[i + 1].input {
            seg = i;
            break;
        }
    }

    let h = points[seg + 1].input - points[seg].input;
    if h.abs() < f64::EPSILON {
        return points[seg].output;
    }

    let t = (x - points[seg].input) / h;
    let t2 = t * t;
    let t3 = t2 * t;

    // Hermite basis functions
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    h00 * points[seg].output
        + h10 * h * m[seg]
        + h01 * points[seg + 1].output
        + h11 * h * m[seg + 1]
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
            osc_output_address: String::new(),
            osc_transform: None,
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
            osc_output_address: String::new(),
            osc_transform: None,
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

    // --- OSC→OSC transform tests ---

    fn make_transform(curve: TransformCurve, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> OscTransform {
        OscTransform {
            curve,
            input_min: in_min,
            input_max: in_max,
            output_min: out_min,
            output_max: out_max,
            calibration_points: vec![],
        }
    }

    #[test]
    fn test_transform_linear_0_to_1() {
        let t = make_transform(TransformCurve::Linear, 0.0, 100.0, 0.0, 1.0);
        assert!((transform_value(0.0, &t) - 0.0).abs() < 1e-9);
        assert!((transform_value(50.0, &t) - 0.5).abs() < 1e-9);
        assert!((transform_value(100.0, &t) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_transform_linear_clamping() {
        let t = make_transform(TransformCurve::Linear, 0.0, 100.0, 0.0, 1.0);
        assert!((transform_value(-10.0, &t) - 0.0).abs() < 1e-9);
        assert!((transform_value(200.0, &t) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_transform_linear_negative_input_range() {
        let t = make_transform(TransformCurve::Linear, -90.0, 10.0, 0.0, 1.0);
        assert!((transform_value(-90.0, &t) - 0.0).abs() < 1e-9);
        assert!((transform_value(10.0, &t) - 1.0).abs() < 1e-9);
        assert!((transform_value(-40.0, &t) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_transform_linear_reversed_output() {
        let t = make_transform(TransformCurve::Linear, 0.0, 100.0, 1.0, 0.0);
        assert!((transform_value(0.0, &t) - 1.0).abs() < 1e-9);
        assert!((transform_value(100.0, &t) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_transform_log_endpoints() {
        let t = make_transform(TransformCurve::Logarithmic, -90.0, 10.0, 0.0, 1.0);
        let at_min = transform_value(-90.0, &t);
        let at_max = transform_value(10.0, &t);
        assert!((at_min - 0.0).abs() < 1e-9, "Min should map to 0.0, got {}", at_min);
        assert!((at_max - 1.0).abs() < 1e-9, "Max should map to 1.0, got {}", at_max);
    }

    #[test]
    fn test_transform_log_bottom_compressed() {
        // Bottom third of input range should produce a small output (compressed)
        let t = make_transform(TransformCurve::Logarithmic, 0.0, 100.0, 0.0, 1.0);
        let at_33 = transform_value(33.3, &t);
        assert!(at_33 < 0.1, "Bottom third should be compressed, got {}", at_33);
    }

    #[test]
    fn test_transform_log_top_expanded() {
        // Top portion of input should have more resolution
        let t = make_transform(TransformCurve::Logarithmic, 0.0, 100.0, 0.0, 1.0);
        let at_90 = transform_value(90.0, &t);
        assert!(at_90 > 0.7, "90% input should map high with log curve, got {}", at_90);
    }

    #[test]
    fn test_transform_log_fader_style() {
        // Simulate a dB fader: -90 to 0, mapped to 0..1
        // 0 dB (max of range) should be 1.0
        // -45 dB (midpoint) should be low due to log compression
        let t = make_transform(TransformCurve::Logarithmic, -90.0, 0.0, 0.0, 1.0);
        let at_0db = transform_value(0.0, &t);
        let at_mid = transform_value(-45.0, &t);
        assert!((at_0db - 1.0).abs() < 1e-9);
        assert!(at_mid < 0.2, "Midpoint of dB range should be compressed low, got {}", at_mid);
    }

    #[test]
    fn test_transform_logarithmic() {
        let t = make_transform(TransformCurve::Logarithmic, 0.0, 100.0, 0.0, 1.0);
        let at_0 = transform_value(0.0, &t);
        let at_50 = transform_value(50.0, &t);
        let at_100 = transform_value(100.0, &t);
        assert!((at_0 - 0.0).abs() < 1e-9);
        assert!((at_100 - 1.0).abs() < 1e-9);
        // Log/power curve: midpoint should map lower than 0.5 (compressed at bottom)
        assert!(at_50 < 0.2, "Log curve at 50% input should be < 0.2, got {}", at_50);
    }

    #[test]
    fn test_transform_zero_range_input() {
        let t = make_transform(TransformCurve::Linear, 5.0, 5.0, 0.0, 1.0);
        assert!((transform_value(5.0, &t) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_apply_transform_preserves_type() {
        let t = make_transform(TransformCurve::Linear, 0.0, 100.0, 0.0, 1.0);
        match apply_transform(&OscArgValue::Float(50.0), &t) {
            OscArgValue::Float(f) => assert!((f - 0.5).abs() < 1e-6),
            _ => panic!("Expected Float"),
        }
        match apply_transform(&OscArgValue::Int(50), &t) {
            OscArgValue::Int(i) => assert_eq!(i, 1), // 0.5 rounds to 1 for 0-1 range
            _ => panic!("Expected Int"),
        }
        let s = OscArgValue::String("hello".to_string());
        match apply_transform(&s, &t) {
            OscArgValue::String(ref val) => assert_eq!(val, "hello"),
            _ => panic!("Expected String passthrough"),
        }
    }

    #[test]
    fn test_apply_transform_int_larger_range() {
        // Int with output range 0-127, input 0-100
        let t = make_transform(TransformCurve::Linear, 0.0, 100.0, 0.0, 127.0);
        match apply_transform(&OscArgValue::Int(100), &t) {
            OscArgValue::Int(i) => assert_eq!(i, 127),
            _ => panic!("Expected Int"),
        }
        match apply_transform(&OscArgValue::Int(50), &t) {
            OscArgValue::Int(i) => assert_eq!(i, 64), // 63.5 rounds to 64
            _ => panic!("Expected Int"),
        }
    }

    // --- Calibration interpolation tests ---

    fn make_calibration_points() -> Vec<CalibrationPoint> {
        // Simulates the user's real-world scenario:
        // -100 dB → 0.0, 0 dB → 0.5, +12 dB → 1.0
        vec![
            CalibrationPoint { input: -100.0, output: 0.0 },
            CalibrationPoint { input: 0.0, output: 0.5 },
            CalibrationPoint { input: 12.0, output: 1.0 },
        ]
    }

    #[test]
    fn test_calibrated_exact_points() {
        let points = make_calibration_points();
        assert!((calibrated_interpolate(-100.0, &points) - 0.0).abs() < 1e-9);
        assert!((calibrated_interpolate(0.0, &points) - 0.5).abs() < 1e-9);
        assert!((calibrated_interpolate(12.0, &points) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_calibrated_interpolation_between_points() {
        let points = make_calibration_points();
        // Monotone cubic: smooth curve through (-100,0), (0,0.5), (12,1.0)
        // -50 is midpoint of the long shallow segment — value should be
        // between 0 and 0.5, monotonically increasing
        let at_minus50 = calibrated_interpolate(-50.0, &points);
        assert!(at_minus50 > 0.0 && at_minus50 < 0.5,
            "-50 should be between 0 and 0.5, got {}", at_minus50);

        let at_plus6 = calibrated_interpolate(6.0, &points);
        assert!(at_plus6 > 0.5 && at_plus6 < 1.0,
            "+6 should be between 0.5 and 1.0, got {}", at_plus6);

        // Monotonicity: values should be strictly increasing
        let at_minus75 = calibrated_interpolate(-75.0, &points);
        let at_minus25 = calibrated_interpolate(-25.0, &points);
        assert!(at_minus75 < at_minus50, "Should be monotone");
        assert!(at_minus50 < at_minus25, "Should be monotone");
        assert!(at_minus25 < at_plus6, "Should be monotone");
    }

    #[test]
    fn test_calibrated_clamping() {
        let points = make_calibration_points();
        // Below min → clamp to first output
        assert!((calibrated_interpolate(-200.0, &points) - 0.0).abs() < 1e-9);
        // Above max → clamp to last output
        assert!((calibrated_interpolate(50.0, &points) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_calibrated_empty_points() {
        let points: Vec<CalibrationPoint> = vec![];
        assert!((calibrated_interpolate(5.0, &points) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_calibrated_single_point() {
        let points = vec![CalibrationPoint { input: 0.0, output: 0.5 }];
        assert!((calibrated_interpolate(0.0, &points) - 0.5).abs() < 1e-9);
        assert!((calibrated_interpolate(100.0, &points) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_calibrated_five_point() {
        // More realistic 5-point calibration
        let points = vec![
            CalibrationPoint { input: -100.0, output: 0.0 },
            CalibrationPoint { input: -40.0, output: 0.25 },
            CalibrationPoint { input: -10.0, output: 0.5 },
            CalibrationPoint { input: 0.0, output: 0.75 },
            CalibrationPoint { input: 12.0, output: 1.0 },
        ];
        // Exact points — spline must pass through them
        assert!((calibrated_interpolate(-100.0, &points) - 0.0).abs() < 1e-6);
        assert!((calibrated_interpolate(-40.0, &points) - 0.25).abs() < 1e-6);
        assert!((calibrated_interpolate(-10.0, &points) - 0.5).abs() < 1e-6);
        assert!((calibrated_interpolate(0.0, &points) - 0.75).abs() < 1e-6);
        assert!((calibrated_interpolate(12.0, &points) - 1.0).abs() < 1e-6);
        // Between points: spline should be smooth and in range
        let result = calibrated_interpolate(-25.0, &points);
        assert!(result > 0.3 && result < 0.45, "Between -40 and -10, got {}", result);
    }

    #[test]
    fn test_calibrated_via_apply_transform() {
        // Test that calibrated works end-to-end through apply_transform
        let t = OscTransform {
            curve: TransformCurve::Calibrated,
            input_min: 0.0,
            input_max: 0.0,
            output_min: 0.0,
            output_max: 0.0,
            calibration_points: make_calibration_points(),
        };
        // Exact calibration points must be hit precisely
        match apply_transform(&OscArgValue::Float(0.0), &t) {
            OscArgValue::Float(f) => assert!((f - 0.5).abs() < 1e-4, "0 dB should be 0.5, got {}", f),
            _ => panic!("Expected Float"),
        }
        match apply_transform(&OscArgValue::Float(-100.0), &t) {
            OscArgValue::Float(f) => assert!((f - 0.0).abs() < 1e-4, "-100 dB should be 0.0, got {}", f),
            _ => panic!("Expected Float"),
        }
        match apply_transform(&OscArgValue::Float(12.0), &t) {
            OscArgValue::Float(f) => assert!((f - 1.0).abs() < 1e-4, "+12 dB should be 1.0, got {}", f),
            _ => panic!("Expected Float"),
        }
    }
}
