import { useState } from "react";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ValidatedInput } from "@/components/ui/validated-input";
import { Button } from "@/components/ui/button";
import { isValidOscAddress } from "@/lib/midi";
import { validateOscFloat } from "@/lib/validators";
import { CalibrationWizard } from "@/components/mapping/CalibrationWizard";
import { Copy, ClipboardPaste, Crosshair } from "lucide-react";
import { toast } from "sonner";
import type {
  Mapping,
  TransformCurve,
  OscTransform,
  OscOutputType,
  CalibrationPoint,
} from "@/types";

const defaultTransform: OscTransform = {
  curve: "linear",
  input_min: 0,
  input_max: 1,
  output_min: 0,
  output_max: 1,
  calibration_points: [],
  output_type: "auto",
  smoothing: 1,
};

interface OscToOscFieldsProps {
  mapping: Mapping;
  onChange: (mapping: Mapping) => void;
}

type CurveOption = TransformCurve | "none";

const curveGlyphPaths: Record<CurveOption, string> = {
  none: "M 2 8 L 13 8 M 10 5 L 13 8 L 10 11",
  linear: "M 2 14 L 14 2",
  logarithmic: "M 2 14 C 9 14, 11 12, 14 2",
  logarithmic_inverse: "M 2 14 C 5 4, 7 2, 14 2",
  calibrated: "M 2 14 L 6 9 L 9 11 L 14 2",
};

function CurveGlyph({ kind }: { kind: CurveOption }) {
  return (
    <svg
      viewBox="0 0 16 16"
      width="14"
      height="14"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.25"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="shrink-0"
      aria-hidden
    >
      <path d={curveGlyphPaths[kind]} />
    </svg>
  );
}

export function OscToOscFields({ mapping, onChange }: OscToOscFieldsProps) {
  const [wizardOpen, setWizardOpen] = useState(false);

  const inputInvalid =
    mapping.osc_address.length > 0 && !isValidOscAddress(mapping.osc_address);
  const outputInvalid =
    mapping.osc_output_address.length > 0 &&
    !isValidOscAddress(mapping.osc_output_address);

  const curveValue: CurveOption = mapping.osc_transform?.curve ?? "none";
  const isCalibrated = curveValue === "calibrated";
  const showRanges =
    mapping.osc_transform !== null && !isCalibrated;
  const showOutputType =
    mapping.osc_transform !== null && !isCalibrated;

  const handleCurveChange = (v: string) => {
    if (v === "none") {
      onChange({ ...mapping, osc_transform: null });
    } else {
      const curve = v as TransformCurve;
      const existing = mapping.osc_transform;
      onChange({
        ...mapping,
        osc_transform: existing
          ? { ...existing, curve }
          : { ...defaultTransform, curve },
      });
    }
  };

  const updateTransformField = (field: keyof OscTransform, value: number) => {
    const t = mapping.osc_transform ?? defaultTransform;
    onChange({
      ...mapping,
      osc_transform: { ...t, [field]: value },
    });
  };

  const updateOutputType = (output_type: OscOutputType) => {
    const t = mapping.osc_transform ?? defaultTransform;
    onChange({
      ...mapping,
      osc_transform: { ...t, output_type },
    });
  };

  const handleTransformChange = (next: OscTransform) => {
    onChange({ ...mapping, osc_transform: next });
  };

  const handleCopyCalibration = async () => {
    const points = mapping.osc_transform?.calibration_points;
    if (!points || points.length === 0) {
      toast.error("No calibration data to copy");
      return;
    }
    try {
      await navigator.clipboard.writeText(
        JSON.stringify({ conduit_calibration: points }),
      );
      toast.success("Calibration copied to clipboard");
    } catch {
      toast.error("Failed to copy to clipboard");
    }
  };

  const handlePasteCalibration = async () => {
    try {
      const text = await navigator.clipboard.readText();
      const data = JSON.parse(text);
      if (!data.conduit_calibration || !Array.isArray(data.conduit_calibration)) {
        toast.error("Clipboard does not contain calibration data");
        return;
      }
      const points: CalibrationPoint[] = data.conduit_calibration;
      // Basic validation
      if (
        !points.every(
          (p) => typeof p.input === "number" && typeof p.output === "number",
        )
      ) {
        toast.error("Invalid calibration data format");
        return;
      }
      onChange({
        ...mapping,
        osc_transform: {
          ...(mapping.osc_transform ?? defaultTransform),
          curve: "calibrated",
          calibration_points: points,
        },
      });
      toast.success(`Calibration pasted (${points.length} points)`);
    } catch {
      toast.error("No valid calibration data in clipboard");
    }
  };

  const calibrationPointCount =
    mapping.osc_transform?.calibration_points?.length ?? 0;

  return (
    <>
      <div className="flex items-center gap-1 flex-wrap">
        <Input
          className={`h-9 text-xs w-52 font-mono ${inputInvalid ? "border-red-500" : ""}`}
          placeholder="/input/address"
          value={mapping.osc_address}
          onChange={(e) =>
            onChange({ ...mapping, osc_address: e.target.value })
          }
          title={inputInvalid ? "OSC address must start with /" : ""}
        />
        <Input
          className={`h-9 text-xs w-52 font-mono ${outputInvalid ? "border-red-500" : ""}`}
          placeholder="/output/address"
          value={mapping.osc_output_address}
          onChange={(e) =>
            onChange({ ...mapping, osc_output_address: e.target.value })
          }
          title={outputInvalid ? "OSC address must start with /" : ""}
        />
        <Select value={curveValue} onValueChange={handleCurveChange}>
          <SelectTrigger className="h-7 text-xs w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="none">
              <span className="flex items-center gap-2">
                <CurveGlyph kind="none" />
                <span>Passthrough</span>
              </span>
            </SelectItem>
            <SelectItem value="linear">
              <span className="flex items-center gap-2">
                <CurveGlyph kind="linear" />
                <span>Linear</span>
              </span>
            </SelectItem>
            <SelectItem value="logarithmic">
              <span className="flex items-center gap-2">
                <CurveGlyph kind="logarithmic" />
                <span>Logarithmic</span>
              </span>
            </SelectItem>
            <SelectItem value="logarithmic_inverse">
              <span className="flex items-center gap-2">
                <CurveGlyph kind="logarithmic_inverse" />
                <span>Inverse Log</span>
              </span>
            </SelectItem>
            <SelectItem value="calibrated">
              <span className="flex items-center gap-2">
                <CurveGlyph kind="calibrated" />
                <span>Calibrated</span>
              </span>
            </SelectItem>
          </SelectContent>
        </Select>
        {showRanges && (
          <>
            <span className="text-xs text-muted-foreground">In</span>
            <ValidatedInput
              inputMode="decimal"
              className="h-9 text-xs w-16 font-mono"
              value={mapping.osc_transform!.input_min}
              validate={validateOscFloat}
              errorMessage="Must be a number"
              onCommit={(v) => updateTransformField("input_min", v)}
              placeholder="min"
            />
            <ValidatedInput
              inputMode="decimal"
              className="h-9 text-xs w-16 font-mono"
              value={mapping.osc_transform!.input_max}
              validate={validateOscFloat}
              errorMessage="Must be a number"
              onCommit={(v) => updateTransformField("input_max", v)}
              placeholder="max"
            />
            <span className="text-xs text-muted-foreground">Out</span>
            <ValidatedInput
              inputMode="decimal"
              className="h-9 text-xs w-16 font-mono"
              value={mapping.osc_transform!.output_min}
              validate={validateOscFloat}
              errorMessage="Must be a number"
              onCommit={(v) => updateTransformField("output_min", v)}
              placeholder="min"
            />
            <ValidatedInput
              inputMode="decimal"
              className="h-9 text-xs w-16 font-mono"
              value={mapping.osc_transform!.output_max}
              validate={validateOscFloat}
              errorMessage="Must be a number"
              onCommit={(v) => updateTransformField("output_max", v)}
              placeholder="max"
            />
          </>
        )}
        {showOutputType && (
          <Select
            value={mapping.osc_transform!.output_type}
            onValueChange={(v) => updateOutputType(v as OscOutputType)}
          >
            <SelectTrigger className="h-7 text-xs w-24" title="Output value type">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="auto">Auto</SelectItem>
              <SelectItem value="int">Int</SelectItem>
              <SelectItem value="float">Float</SelectItem>
            </SelectContent>
          </Select>
        )}
        {isCalibrated && (
          <>
            <Button
              variant="outline"
              size="sm"
              className="h-7 text-xs gap-1"
              onClick={() => setWizardOpen(true)}
            >
              <Crosshair className="h-3 w-3" />
              {calibrationPointCount > 0
                ? `${calibrationPointCount} pts`
                : "Calibrate"}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              className="h-7 w-7 p-0"
              onClick={handleCopyCalibration}
              title="Copy calibration to clipboard"
            >
              <Copy className="h-3 w-3" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              className="h-7 w-7 p-0"
              onClick={handlePasteCalibration}
              title="Paste calibration from clipboard"
            >
              <ClipboardPaste className="h-3 w-3" />
            </Button>
          </>
        )}
      </div>
      <CalibrationWizard
        open={wizardOpen}
        onClose={() => setWizardOpen(false)}
        outputAddress={mapping.osc_output_address}
        transform={mapping.osc_transform ?? defaultTransform}
        onChange={handleTransformChange}
      />
    </>
  );
}
