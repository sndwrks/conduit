import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import type { CalibrationPoint } from "@/types";

const TEST_VALUES = [0.0, 0.25, 0.5, 0.75, 1.0];

interface CalibrationWizardProps {
  open: boolean;
  onClose: () => void;
  outputAddress: string;
  onComplete: (points: CalibrationPoint[]) => void;
}

export function CalibrationWizard({
  open,
  onClose,
  outputAddress,
  onComplete,
}: CalibrationWizardProps) {
  const [step, setStep] = useState(0);
  const [results, setResults] = useState<(number | null)[]>(
    TEST_VALUES.map(() => null),
  );
  const [inputValue, setInputValue] = useState("");
  const [sending, setSending] = useState(false);
  const [sent, setSent] = useState(false);

  const currentTestValue = TEST_VALUES[step];
  const isLastStep = step === TEST_VALUES.length - 1;

  const reset = () => {
    setStep(0);
    setResults(TEST_VALUES.map(() => null));
    setInputValue("");
    setSending(false);
    setSent(false);
  };

  const handleSend = async () => {
    if (!outputAddress) return;
    setSending(true);
    try {
      await invoke("send_osc_test_value", {
        address: outputAddress,
        value: currentTestValue,
      });
      setSent(true);
    } catch (e) {
      console.error("Failed to send test value:", e);
    } finally {
      setSending(false);
    }
  };

  const handleRecord = () => {
    const parsed = parseFloat(inputValue);
    if (isNaN(parsed)) return;

    const newResults = [...results];
    newResults[step] = parsed;
    setResults(newResults);

    if (isLastStep) {
      // Build calibration points and finish
      const points: CalibrationPoint[] = newResults
        .map((input, i) => ({
          input: input!,
          output: TEST_VALUES[i],
        }))
        .filter((p) => p.input !== null);

      // Sort by input value
      points.sort((a, b) => a.input - b.input);
      onComplete(points);
      reset();
    } else {
      setStep(step + 1);
      setInputValue("");
      setSent(false);
    }
  };

  const handleClose = () => {
    reset();
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && handleClose()}>
      <DialogContent showCloseButton={false}>
        <DialogHeader>
          <DialogTitle>Calibrate Transform</DialogTitle>
          <DialogDescription>
            Step {step + 1} of {TEST_VALUES.length} — Sending test values to{" "}
            <span className="font-mono text-foreground">{outputAddress || "(no address)"}</span>
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="flex items-center gap-3">
            <div className="text-sm text-muted-foreground w-28">
              Test value:
            </div>
            <div className="font-mono text-lg">{currentTestValue.toFixed(2)}</div>
            <Button
              size="sm"
              variant="outline"
              onClick={handleSend}
              disabled={sending || !outputAddress}
            >
              {sent ? "Sent" : "Send"}
            </Button>
          </div>

          <div className="flex items-center gap-3">
            <div className="text-sm text-muted-foreground w-28">
              Target shows:
            </div>
            <Input
              className="h-9 text-xs w-32 font-mono"
              placeholder="e.g. -100, 0, +12"
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleRecord();
              }}
              inputMode="decimal"
            />
            <span className="text-xs text-muted-foreground">
              (the value the plugin displays)
            </span>
          </div>

          {/* Show completed steps */}
          {results.some((r) => r !== null) && (
            <div className="border rounded p-2 space-y-1">
              <div className="text-xs text-muted-foreground mb-1">
                Recorded points:
              </div>
              {results.map((r, i) =>
                r !== null ? (
                  <div key={i} className="text-xs font-mono flex gap-4">
                    <span className="text-muted-foreground w-20">
                      Sent {TEST_VALUES[i].toFixed(2)}
                    </span>
                    <span>= {r}</span>
                  </div>
                ) : null,
              )}
            </div>
          )}
        </div>

        <p className="text-xs text-muted-foreground">
          Note: With 5 calibration points, values between samples are
          interpolated and may be approximate — particularly toward the
          extremes of the range.
        </p>

        <DialogFooter>
          <Button variant="outline" onClick={handleClose}>
            Cancel
          </Button>
          <Button
            onClick={handleRecord}
            disabled={inputValue === "" || isNaN(parseFloat(inputValue))}
          >
            {isLastStep ? "Finish" : "Next"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
