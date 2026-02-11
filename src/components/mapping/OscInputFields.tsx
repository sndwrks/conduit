import { Input } from "@/components/ui/input";
import { isValidOscAddress } from "@/lib/midi";
import type { Mapping } from "@/types";

interface OscInputFieldsProps {
  mapping: Mapping;
  onChange: (mapping: Mapping) => void;
}

export function OscInputFields({ mapping, onChange }: OscInputFieldsProps) {
  const invalid =
    mapping.osc_address.length > 0 && !isValidOscAddress(mapping.osc_address);

  return (
    <div className="flex items-center gap-1">
      <Input
        className={`h-9 text-xs w-40 font-mono ${invalid ? "border-red-500" : ""}`}
        placeholder="/osc/address"
        value={mapping.osc_address}
        onChange={(e) => onChange({ ...mapping, osc_address: e.target.value })}
        title={invalid ? "OSC address must start with /" : ""}
      />
    </div>
  );
}
