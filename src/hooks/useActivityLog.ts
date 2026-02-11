import { useState, useEffect, useCallback, useRef } from "react";
import { typedListen } from "@/lib/events";
import type { MappingActivityEvent, UnmatchedMessageEvent } from "@/types";

export interface LogEntry {
  id: number;
  timestamp: string;
  type: "activity" | "unmatched";
  inputProtocol?: string;
  inputDisplay?: string;
  outputProtocol?: string;
  outputDisplay?: string;
  protocol?: string;
  display?: string;
}

const MAX_ENTRIES = 500;

export function useActivityLog() {
  const [entries, setEntries] = useState<LogEntry[]>([]);
  const [paused, setPaused] = useState(false);
  const pausedRef = useRef(false);
  const counterRef = useRef(0);

  useEffect(() => {
    pausedRef.current = paused;
  }, [paused]);

  useEffect(() => {
    const unlisten1 = typedListen<MappingActivityEvent>(
      "mapping-activity",
      (payload) => {
        if (pausedRef.current) return;
        const entry: LogEntry = {
          id: counterRef.current++,
          timestamp: payload.timestamp,
          type: "activity",
          inputProtocol: payload.input_protocol,
          inputDisplay: payload.input_display,
          outputProtocol: payload.output_protocol,
          outputDisplay: payload.output_display,
        };
        setEntries((prev) => [...prev.slice(-(MAX_ENTRIES - 1)), entry]);
      },
    );

    const unlisten2 = typedListen<UnmatchedMessageEvent>(
      "unmatched-message",
      (payload) => {
        if (pausedRef.current) return;
        const entry: LogEntry = {
          id: counterRef.current++,
          timestamp: payload.timestamp,
          type: "unmatched",
          protocol: payload.protocol,
          display: payload.display,
        };
        setEntries((prev) => [...prev.slice(-(MAX_ENTRIES - 1)), entry]);
      },
    );

    return () => {
      unlisten1.then((fn) => fn());
      unlisten2.then((fn) => fn());
    };
  }, []);

  const clear = useCallback(() => setEntries([]), []);
  const togglePause = useCallback(() => setPaused((p) => !p), []);

  return { entries, paused, clear, togglePause };
}
