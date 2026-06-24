import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { ValidatedInput } from "@/components/ui/validated-input";
import { validateOscFloat } from "@/lib/validators";
import {
  interpolateCalibrated,
  linearDistribute,
} from "@/components/mapping/curveMath";
import type { CalibrationPoint, OscOutputType, OscTransform } from "@/types";

const POINTS_COUNT = 5;
const GRAPH_W = 360;
const GRAPH_H = 110;
const GRAPH_PAD_X = 6;
const GRAPH_PAD_Y = 3;

interface CalibrationWizardProps {
  open: boolean;
  onClose: () => void;
  outputAddress: string;
  transform: OscTransform;
  onChange: (transform: OscTransform) => void;
}

function buildPoints(
  inputMin: number,
  inputMax: number,
  outputs: number[],
): CalibrationPoint[] {
  const inputs = linearDistribute(inputMin, inputMax, POINTS_COUNT);
  return inputs.map((input, i) => ({ input, output: outputs[i] ?? 0 }));
}

function pointsAreLinear(
  points: CalibrationPoint[],
  inputMin: number,
  inputMax: number,
): boolean {
  if (points.length !== POINTS_COUNT) return false;
  const expected = linearDistribute(inputMin, inputMax, POINTS_COUNT);
  return points.every((p, i) => Math.abs(p.input - expected[i]) < 1e-9);
}

export function CalibrationWizard({
  open,
  onClose,
  outputAddress,
  transform,
  onChange,
}: CalibrationWizardProps) {
  const inputs = useMemo(
    () => linearDistribute(transform.input_min, transform.input_max, POINTS_COUNT),
    [transform.input_min, transform.input_max],
  );

  const outputs = useMemo(() => {
    const arr: number[] = [];
    for (let i = 0; i < POINTS_COUNT; i++) {
      arr.push(transform.calibration_points[i]?.output ?? 0);
    }
    return arr;
  }, [transform.calibration_points]);

  // On open, if existing calibration_points don't align with the linear
  // distribution implied by input_min/max, migrate: derive range from points
  // (preserves data from the legacy wizard) and re-emit with linear inputs.
  useEffect(() => {
    if (!open) return;
    if (pointsAreLinear(transform.calibration_points, transform.input_min, transform.input_max)) {
      return;
    }
    let newMin = transform.input_min;
    let newMax = transform.input_max;
    if (transform.calibration_points.length >= 2) {
      const sorted = [...transform.calibration_points].sort((a, b) => a.input - b.input);
      newMin = sorted[0].input;
      newMax = sorted[sorted.length - 1].input;
    }
    const oldOutputs: number[] = [];
    for (let i = 0; i < POINTS_COUNT; i++) {
      oldOutputs.push(transform.calibration_points[i]?.output ?? 0);
    }
    onChange({
      ...transform,
      input_min: newMin,
      input_max: newMax,
      calibration_points: buildPoints(newMin, newMax, oldOutputs),
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const updateRange = (field: "input_min" | "input_max", value: number) => {
    const newMin = field === "input_min" ? value : transform.input_min;
    const newMax = field === "input_max" ? value : transform.input_max;
    onChange({
      ...transform,
      input_min: newMin,
      input_max: newMax,
      calibration_points: buildPoints(newMin, newMax, outputs),
    });
  };

  const updateOutput = (index: number, value: number) => {
    const next = outputs.slice();
    next[index] = value;
    onChange({
      ...transform,
      calibration_points: buildPoints(transform.input_min, transform.input_max, next),
    });
  };

  const updateSmoothing = (value: number) => {
    onChange({ ...transform, smoothing: value });
  };

  const updateOutputType = (value: OscOutputType) => {
    onChange({ ...transform, output_type: value });
  };

  const [sending, setSending] = useState<{ row: number; side: "in" | "out" } | null>(
    null,
  );
  const handleSend = async (rowIndex: number, side: "in" | "out") => {
    if (!outputAddress) return;
    setSending({ row: rowIndex, side });
    try {
      await invoke("send_osc_test_value", {
        address: outputAddress,
        value: side === "in" ? inputs[rowIndex] : outputs[rowIndex],
      });
    } catch (e) {
      console.error("Failed to send test value:", e);
    } finally {
      setSending(null);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent
        showCloseButton={false}
        className="sm:max-w-2xl max-h-[85vh] overflow-y-auto"
      >
        <DialogHeader>
          <DialogTitle>Calibrate Transform</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2">
          {/* Top row: input range + output type */}
          <div className="flex items-center gap-2 flex-wrap">
            <span className="text-xs text-muted-foreground">Input</span>
            <ValidatedInput
              inputMode="decimal"
              className="h-9 text-xs w-20 font-mono"
              value={transform.input_min}
              validate={validateOscFloat}
              errorMessage="Must be a number"
              onCommit={(v) => updateRange("input_min", v)}
              placeholder="min"
            />
            <span className="text-xs text-muted-foreground">to</span>
            <ValidatedInput
              inputMode="decimal"
              className="h-9 text-xs w-20 font-mono"
              value={transform.input_max}
              validate={validateOscFloat}
              errorMessage="Must be a number"
              onCommit={(v) => updateRange("input_max", v)}
              placeholder="max"
            />
            <span className="text-xs text-muted-foreground ml-4">Output type</span>
            <Select
              value={transform.output_type}
              onValueChange={(v) => updateOutputType(v as OscOutputType)}
            >
              <SelectTrigger className="h-7 text-xs w-28">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="auto">Auto</SelectItem>
                <SelectItem value="int">Int</SelectItem>
                <SelectItem value="float">Float</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Two columns: rows on the left, graph + smoothing + note on the right */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="grid grid-cols-[auto_auto_1fr_auto] gap-x-2 gap-y-1.5 items-center min-w-0 self-start">
              <div className="text-xs text-muted-foreground">In</div>
              <div />
              <div className="text-xs text-muted-foreground">Out</div>
              <div />
              {inputs.map((inputVal, i) => (
                <FiveRow
                  key={i}
                  inputVal={inputVal}
                  outputVal={outputs[i]}
                  onOutputChange={(v) => updateOutput(i, v)}
                  onSendIn={() => handleSend(i, "in")}
                  onSendOut={() => handleSend(i, "out")}
                  sendingIn={sending?.row === i && sending.side === "in"}
                  sendingOut={sending?.row === i && sending.side === "out"}
                  disabled={!outputAddress}
                />
              ))}
            </div>

            <div className="space-y-3 min-w-0 mt-5">
              <CurveGraph
                inputMin={transform.input_min}
                inputMax={transform.input_max}
                points={transform.calibration_points}
                smoothing={transform.smoothing}
              />
              <div className="flex items-center gap-3">
                <span className="text-xs text-muted-foreground w-20">Smoothing</span>
                <Slider
                  className="flex-1"
                  min={0}
                  max={100}
                  step={1}
                  value={[Math.round(transform.smoothing * 100)]}
                  onValueChange={(v) => updateSmoothing((v[0] ?? 0) / 100)}
                />
                <span className="text-xs font-mono w-10 text-right tabular-nums">
                  {Math.round(transform.smoothing * 100)}%
                </span>
              </div>
              <p className="text-xs text-muted-foreground">
                Note: With 5 calibration points, values between samples are
                interpolated and may be approximate — particularly toward the
                extremes of the range.
              </p>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button onClick={onClose}>Done</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

interface FiveRowProps {
  inputVal: number;
  outputVal: number;
  onOutputChange: (v: number) => void;
  onSendIn: () => void;
  onSendOut: () => void;
  sendingIn: boolean;
  sendingOut: boolean;
  disabled: boolean;
}

function FiveRow({
  inputVal,
  outputVal,
  onOutputChange,
  onSendIn,
  onSendOut,
  sendingIn,
  sendingOut,
  disabled,
}: FiveRowProps) {
  return (
    <>
      <div className="font-mono text-xs text-muted-foreground tabular-nums">
        {formatNum(inputVal)}
      </div>
      <Button
        size="sm"
        variant="outline"
        className="h-8 text-xs px-2"
        onClick={onSendIn}
        disabled={disabled || sendingIn}
        title={disabled ? "Set an output address first" : "Send this input value"}
      >
        {sendingIn ? "…" : "Send"}
      </Button>
      <ValidatedInput
        inputMode="decimal"
        className="h-8 text-xs font-mono"
        value={outputVal}
        validate={validateOscFloat}
        errorMessage="Must be a number"
        onCommit={onOutputChange}
        placeholder="0"
      />
      <Button
        size="sm"
        variant="outline"
        className="h-8 text-xs px-2"
        onClick={onSendOut}
        disabled={disabled || sendingOut}
        title={disabled ? "Set an output address first" : "Send this output value"}
      >
        {sendingOut ? "…" : "Send"}
      </Button>
    </>
  );
}

function formatNum(n: number): string {
  if (!Number.isFinite(n)) return "—";
  const abs = Math.abs(n);
  if (abs !== 0 && (abs < 0.01 || abs >= 10000)) {
    return n.toExponential(2);
  }
  return Number.isInteger(n) ? n.toFixed(0) : n.toFixed(2);
}

interface CurveGraphProps {
  inputMin: number;
  inputMax: number;
  points: CalibrationPoint[];
  smoothing: number;
}

function CurveGraph({ inputMin, inputMax, points, smoothing }: CurveGraphProps) {
  const samples = 96;

  const { pathD, dotCoords, yMin, yMax } = useMemo(() => {
    const yValues = points.map((p) => p.output);
    const yMinLocal = Math.min(0, ...yValues);
    let yMaxLocal = Math.max(1, ...yValues);
    if (yMaxLocal - yMinLocal < 1e-9) {
      yMaxLocal = yMinLocal + 1;
    }
    const xRange = inputMax - inputMin;
    const yRange = yMaxLocal - yMinLocal;
    const innerW = GRAPH_W - GRAPH_PAD_X * 2;
    const innerH = GRAPH_H - GRAPH_PAD_Y * 2;

    const toScreen = (x: number, y: number) => {
      const sx =
        GRAPH_PAD_X +
        (Math.abs(xRange) < 1e-9 ? 0 : ((x - inputMin) / xRange) * innerW);
      const sy =
        GRAPH_PAD_Y + innerH - ((y - yMinLocal) / yRange) * innerH;
      return [sx, sy] as const;
    };

    let d = "";
    if (points.length >= 2 && Math.abs(xRange) > 1e-9) {
      for (let i = 0; i < samples; i++) {
        const t = i / (samples - 1);
        const x = inputMin + t * xRange;
        const y = interpolateCalibrated(x, points, smoothing);
        const [sx, sy] = toScreen(x, y);
        d += (i === 0 ? "M" : "L") + sx.toFixed(2) + "," + sy.toFixed(2) + " ";
      }
    }

    const dots = points.map((p) => toScreen(p.input, p.output));
    return { pathD: d.trim(), dotCoords: dots, yMin: yMinLocal, yMax: yMaxLocal };
  }, [inputMin, inputMax, points, smoothing]);

  return (
    <div className="border rounded bg-muted/30 px-2 py-2">
      <svg
        viewBox={`0 0 ${GRAPH_W} ${GRAPH_H}`}
        width="100%"
        height={GRAPH_H}
        preserveAspectRatio="none"
        className="block"
        style={{ overflow: "visible" }}
      >
        {/* Frame */}
        <rect
          x={GRAPH_PAD_X}
          y={GRAPH_PAD_Y}
          width={GRAPH_W - GRAPH_PAD_X * 2}
          height={GRAPH_H - GRAPH_PAD_Y * 2}
          fill="none"
          stroke="currentColor"
          strokeOpacity={0.15}
          strokeWidth={1}
        />
        {/* Curve */}
        {pathD && (
          <path
            d={pathD}
            fill="none"
            stroke="currentColor"
            strokeWidth={1.5}
            strokeLinecap="round"
            strokeLinejoin="round"
            className="text-primary"
          />
        )}
        {/* Control points */}
        {dotCoords.map(([cx, cy], i) => (
          <circle
            key={i}
            cx={cx}
            cy={cy}
            r={2.5}
            className="fill-primary"
          />
        ))}
      </svg>
      <div className="flex justify-between font-mono text-[10px] text-muted-foreground tabular-nums px-0.5">
        <span>
          y: {formatNum(yMin)}–{formatNum(yMax)}
        </span>
        <span>
          x: {formatNum(inputMin)}–{formatNum(inputMax)}
        </span>
      </div>
    </div>
  );
}
