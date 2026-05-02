import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ValidatedInput } from "@/components/ui/validated-input";
import { midiNoteToName } from "@/lib/midi";
import { validateMidiNote, validateMidiValue } from "@/lib/validators";
import type { Mapping, MidiMessageType, MscCommand, MscCommandFormat } from "@/types";

interface MidiInputFieldsProps {
  mapping: Mapping;
  onChange: (mapping: Mapping) => void;
}

export function MidiInputFields({ mapping, onChange }: MidiInputFieldsProps) {
  const isCC = mapping.midi_message_type === "cc";
  const isPC = mapping.midi_message_type === "program_change";
  const isMsc = mapping.midi_message_type === "msc";
  const velocityMode = mapping.midi_input_velocity !== null ? "exact" : "any";

  return (
    <div className="flex items-center gap-1">
      <Select
        value={mapping.midi_message_type}
        onValueChange={(v) =>
          onChange({ ...mapping, midi_message_type: v as MidiMessageType })
        }
      >
        <SelectTrigger className="h-9 text-xs w-38">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="note_on">Note On</SelectItem>
          <SelectItem value="note_off">Note Off</SelectItem>
          <SelectItem value="cc">CC</SelectItem>
          <SelectItem value="program_change">Program Change</SelectItem>
          <SelectItem value="msc">MSC</SelectItem>
        </SelectContent>
      </Select>
      {isMsc ? (
        <>
          <Select
            value={mapping.msc_command ?? "any"}
            onValueChange={(v) =>
              onChange({
                ...mapping,
                msc_command: v === "any" ? null : (v as MscCommand),
              })
            }
          >
            <SelectTrigger className="h-9 text-xs w-28">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="any">Any Cmd</SelectItem>
              <SelectItem value="go">GO</SelectItem>
              <SelectItem value="stop">STOP</SelectItem>
              <SelectItem value="resume">RESUME</SelectItem>
            </SelectContent>
          </Select>
          <Select
            value={mapping.msc_command_format ?? "any"}
            onValueChange={(v) =>
              onChange({
                ...mapping,
                msc_command_format: v === "any" ? null : (v as MscCommandFormat),
              })
            }
          >
            <SelectTrigger className="h-9 text-xs w-28">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="any">Any Type</SelectItem>
              <SelectItem value="all">All Types</SelectItem>
              <SelectItem value="lighting">Lighting</SelectItem>
              <SelectItem value="sound">Sound</SelectItem>
            </SelectContent>
          </Select>
          <Select
            value={mapping.msc_device_id === null ? "all" : String(mapping.msc_device_id)}
            onValueChange={(v) =>
              onChange({
                ...mapping,
                msc_device_id: v === "all" ? null : parseInt(v),
              })
            }
          >
            <SelectTrigger className="h-9 text-xs w-24">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All IDs</SelectItem>
              {Array.from({ length: 128 }, (_, i) => (
                <SelectItem key={i} value={String(i)}>
                  ID {i}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </>
      ) : (
        <>
          <div className="flex items-center gap-1">
            <ValidatedInput
              className="h-9 text-xs w-18"
              value={mapping.midi_note_or_cc}
              validate={validateMidiNote}
              errorMessage="Note must be 0–127 or a note name (e.g. C3)"
              onCommit={(v) => onChange({ ...mapping, midi_note_or_cc: v })}
            />
            <span className="text-xs text-muted-foreground w-8">
              {isCC ? "CC" : isPC ? "Pgm" : midiNoteToName(mapping.midi_note_or_cc)}
            </span>
          </div>
          {!isPC && (
            <Select
              value={velocityMode}
              onValueChange={(v) =>
                onChange({
                  ...mapping,
                  midi_input_velocity: v === "exact" ? 100 : null,
                })
              }
            >
              <SelectTrigger className="h-9 text-xs w-28">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="any">Vel Any</SelectItem>
                <SelectItem value="exact">Vel Exact</SelectItem>
              </SelectContent>
            </Select>
          )}
          {!isPC && velocityMode === "exact" && (
            <ValidatedInput
              inputMode="numeric"
              className="h-9 text-xs w-18"
              value={mapping.midi_input_velocity ?? 0}
              validate={validateMidiValue}
              errorMessage="Value must be 0–127"
              onCommit={(v) =>
                onChange({ ...mapping, midi_input_velocity: v })
              }
            />
          )}
          <Select
            value={String(mapping.midi_channel)}
            onValueChange={(v) =>
              onChange({ ...mapping, midi_channel: parseInt(v) })
            }
          >
            <SelectTrigger className="h-9 text-xs w-24">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {Array.from({ length: 16 }, (_, i) => (
                <SelectItem key={i + 1} value={String(i + 1)}>
                  Ch {i + 1}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </>
      )}
    </div>
  );
}
