import { Trash2, ArrowRight } from "lucide-react";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { OscInputFields } from "@/components/mapping/OscInputFields";
import { MidiInputFields } from "@/components/mapping/MidiInputFields";
import { MidiOutputFields } from "@/components/mapping/MidiOutputFields";
import { OscOutputFields } from "@/components/mapping/OscOutputFields";
import { cn } from "@/lib/utils";
import type { Mapping, Direction } from "@/types";

interface MappingRowProps {
  mapping: Mapping;
  onChange: (mapping: Mapping) => void;
  onDelete: (id: string) => void;
}

export function MappingRow({ mapping, onChange, onDelete }: MappingRowProps) {
  const isOscToMidi = mapping.direction === "osc_to_midi";

  return (
    <div
      className={cn(
        "flex items-center gap-2 px-3 py-1.5 border-b border-border hover:bg-primary/5",
        !mapping.enabled && "opacity-50",
      )}
    >
      <Switch
        checked={mapping.enabled}
        onCheckedChange={(checked) =>
          onChange({ ...mapping, enabled: checked })
        }
        className="scale-75"
      />
      <Select
        value={mapping.direction}
        onValueChange={(v) =>
          onChange({ ...mapping, direction: v as Direction })
        }
      >
        <SelectTrigger className="h-7 text-xs w-32">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="osc_to_midi">OSC → MIDI</SelectItem>
          <SelectItem value="midi_to_osc">MIDI → OSC</SelectItem>
        </SelectContent>
      </Select>

      <div className="flex items-center gap-2 flex-1 min-w-0">
        {/* Input fields */}
        {isOscToMidi ? (
          <OscInputFields mapping={mapping} onChange={onChange} />
        ) : (
          <MidiInputFields mapping={mapping} onChange={onChange} />
        )}

        <ArrowRight className="h-3 w-3 text-muted-foreground shrink-0" />

        {/* Output fields */}
        {isOscToMidi ? (
          <MidiOutputFields mapping={mapping} onChange={onChange} />
        ) : (
          <OscOutputFields mapping={mapping} onChange={onChange} />
        )}
      </div>

      <div className="shrink-0">
        <AlertDialog>
          <AlertDialogTrigger asChild>
            <Button variant="ghost" size="sm" className="h-7 w-7 p-0">
              <Trash2 className="h-3.5 w-3.5 text-muted-foreground" />
            </Button>
          </AlertDialogTrigger>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>Delete mapping?</AlertDialogTitle>
              <AlertDialogDescription>
                This action cannot be undone.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>Cancel</AlertDialogCancel>
              <AlertDialogAction onClick={() => onDelete(mapping.id)}>
                Delete
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>
    </div>
  );
}
