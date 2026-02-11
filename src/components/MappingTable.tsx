import { Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { MappingRow } from "@/components/MappingRow";
import type { Mapping } from "@/types";

interface MappingTableProps {
  mappings: Mapping[];
  onUpdateMapping: (mapping: Mapping) => void;
  onDeleteMapping: (id: string) => void;
  onAddMapping: () => void;
}

export function MappingTable({
  mappings,
  onUpdateMapping,
  onDeleteMapping,
  onAddMapping,
}: MappingTableProps) {
  return (
    <div className="flex flex-col flex-1 min-h-0">
      <div className="flex-1 overflow-y-auto">
        {mappings.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center space-y-2">
              <p className="text-sm text-muted-foreground">
                No mappings yet
              </p>
              <p className="text-xs text-muted-foreground">
                Add a mapping to start bridging OSC and MIDI
              </p>
            </div>
          </div>
        ) : (
          mappings.map((mapping) => (
            <MappingRow
              key={mapping.id}
              mapping={mapping}
              onChange={onUpdateMapping}
              onDelete={onDeleteMapping}
            />
          ))
        )}
      </div>
      <div className="border-t border-border px-3 py-1.5">
        <Button
          variant="ghost"
          size="sm"
          className="text-xs gap-1"
          onClick={onAddMapping}
        >
          <Plus className="h-3.5 w-3.5" />
          Add Mapping
        </Button>
      </div>
    </div>
  );
}
