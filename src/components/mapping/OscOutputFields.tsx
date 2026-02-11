import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ValidatedInput } from "@/components/ui/validated-input";
import { isValidOscAddress } from "@/lib/midi";
import { validateOscFloat, validateOscInt } from "@/lib/validators";
import type { Mapping, OscArgDef, OscArgType, OscArgSource } from "@/types";
import { Plus, X } from "lucide-react";
import { Button } from "@/components/ui/button";

interface OscOutputFieldsProps {
  mapping: Mapping;
  onChange: (mapping: Mapping) => void;
}

export function OscOutputFields({ mapping, onChange }: OscOutputFieldsProps) {
  const invalid =
    mapping.osc_address.length > 0 && !isValidOscAddress(mapping.osc_address);

  const updateArg = (index: number, arg: OscArgDef) => {
    const args = [...mapping.osc_args];
    args[index] = arg;
    onChange({ ...mapping, osc_args: args });
  };

  const addArg = () => {
    const newArg: OscArgDef = {
      type: "float",
      source: { type: "midi_value" },
    };
    onChange({ ...mapping, osc_args: [...mapping.osc_args, newArg] });
  };

  const removeArg = (index: number) => {
    const args = mapping.osc_args.filter((_, i) => i !== index);
    onChange({ ...mapping, osc_args: args });
  };

  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-center gap-1">
        <Input
          className={`h-9 text-xs w-40 font-mono ${invalid ? "border-red-500" : ""}`}
          placeholder="/osc/address"
          value={mapping.osc_address}
          onChange={(e) => onChange({ ...mapping, osc_address: e.target.value })}
        />
        <Button
          variant="ghost"
          size="sm"
          className="h-7 text-xs gap-0.5"
          onClick={addArg}
        >
          <Plus className="h-3 w-3" />
          Arg
        </Button>
      </div>
      {mapping.osc_args.map((arg, i) => (
        <div key={i} className="flex items-center gap-0.5">
          <Select
            value={arg.type}
            onValueChange={(v) =>
              updateArg(i, { ...arg, type: v as OscArgType })
            }
          >
            <SelectTrigger className="h-7 text-xs w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="float">float</SelectItem>
              <SelectItem value="int">int</SelectItem>
              <SelectItem value="string">string</SelectItem>
            </SelectContent>
          </Select>
          <Select
            value={arg.source.type}
            onValueChange={(v) => {
              let source: OscArgSource;
              if (v === "static") {
                source = { type: "static", value: arg.type === "string" ? "" : 0 };
              } else if (v === "midi_value") {
                source = { type: "midi_value" };
              } else {
                source = { type: "midi_note" };
              }
              updateArg(i, { ...arg, source });
            }}
          >
            <SelectTrigger className="h-7 text-xs w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="midi_value">MIDI Value</SelectItem>
              <SelectItem value="midi_note">MIDI Note</SelectItem>
              <SelectItem value="static">Static</SelectItem>
            </SelectContent>
          </Select>
          {arg.source.type === "static" && arg.type === "string" && (
            <Input
              className="h-9 text-xs w-16 font-mono"
              value={arg.source.value}
              onChange={(e) =>
                updateArg(i, {
                  ...arg,
                  source: { type: "static", value: e.target.value },
                })
              }
            />
          )}
          {arg.source.type === "static" && arg.type !== "string" && (
            <ValidatedInput
              inputMode="decimal"
              className="h-9 text-xs w-16 font-mono"
              value={arg.source.value}
              validate={arg.type === "float" ? validateOscFloat : validateOscInt}
              errorMessage={
                arg.type === "float"
                  ? "Must be a valid number"
                  : "Must be a valid integer"
              }
              onCommit={(v) =>
                updateArg(i, {
                  ...arg,
                  source: { type: "static", value: v },
                })
              }
            />
          )}
          <Button
            variant="ghost"
            size="sm"
            className="h-7 w-7 p-0"
            onClick={() => removeArg(i)}
          >
            <X className="h-3 w-3" />
          </Button>
        </div>
      ))}
    </div>
  );
}
