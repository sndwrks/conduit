import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { typedListen } from "@/lib/events";
import type { MidiPort } from "@/types";

export function useMidi() {
  const [inputs, setInputs] = useState<MidiPort[]>([]);
  const [outputs, setOutputs] = useState<MidiPort[]>([]);

  const refresh = useCallback(async () => {
    const [ins, outs] = await Promise.all([
      invoke<MidiPort[]>("list_midi_inputs"),
      invoke<MidiPort[]>("list_midi_outputs"),
    ]);
    setInputs(ins);
    setOutputs(outs);
  }, []);

  useEffect(() => {
    refresh();
    const unlisten = typedListen("midi-devices-changed", () => {
      refresh();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [refresh]);

  return { inputs, outputs, refresh };
}
