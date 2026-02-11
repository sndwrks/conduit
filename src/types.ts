export interface Settings {
  osc_listen_port: number;
  osc_listen_protocol: OscListenProtocol;
  osc_send_host: string;
  osc_send_port: number;
  osc_send_protocol: OscSendProtocol;
  osc_tcp_send_timeout_ms: number;
  midi_input_port_name: string | null;
  midi_output_port_name: string | null;
  engine_auto_start: boolean;
  launch_on_startup: boolean;
}

export type OscListenProtocol = "udp" | "tcp" | "both";
export type OscSendProtocol = "udp" | "tcp";

export interface Mapping {
  id: string;
  enabled: boolean;
  direction: Direction;
  osc_address: string;
  osc_arg_types: OscArgType[];
  midi_message_type: MidiMessageType;
  midi_channel: number;
  midi_note_or_cc: number;
  midi_velocity_or_value: ValueSource;
  midi_input_velocity: number | null;
  osc_args: OscArgDef[];
}

export type Direction = "osc_to_midi" | "midi_to_osc";
export type MidiMessageType = "note_on" | "note_off" | "cc" | "program_change";
export type OscArgType = "int" | "float" | "string";

export type ValueSource =
  | { type: "static"; value: number }
  | { type: "osc_arg"; index: number };

export interface OscArgDef {
  type: OscArgType;
  source: OscArgSource;
}

export type OscArgSource =
  | { type: "static"; value: number | string }
  | { type: "midi_value" }
  | { type: "midi_note" };

export interface MidiPort {
  name: string;
  index: number;
}

export interface EngineStatus {
  running: boolean;
  error?: string;
}

export interface MappingActivityEvent {
  timestamp: string;
  input_protocol: "osc" | "midi";
  input_display: string;
  output_protocol: "osc" | "midi";
  output_display: string;
  mapping_id: string;
}

export interface UnmatchedMessageEvent {
  timestamp: string;
  protocol: "osc" | "midi";
  display: string;
}

export function defaultMapping(): Mapping {
  return {
    id: "",
    enabled: true,
    direction: "osc_to_midi",
    osc_address: "",
    osc_arg_types: [],
    midi_message_type: "note_on",
    midi_channel: 1,
    midi_note_or_cc: 60,
    midi_velocity_or_value: { type: "static", value: 127 },
    midi_input_velocity: null,
    osc_args: [],
  };
}
