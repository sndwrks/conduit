import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { typedListen } from "@/lib/events";
import type { EngineStatus } from "@/types";

export function useEngine() {
  const [status, setStatus] = useState<EngineStatus>({ running: false });

  useEffect(() => {
    invoke<EngineStatus>("get_engine_status")
      .then(setStatus)
      .catch((e) => {
        console.error(e);
        toast.error("Failed to get engine status");
      });

    const unlisten = typedListen<EngineStatus>("engine-status", setStatus);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const start = useCallback(async () => {
    await invoke("start_engine");
    setStatus({ running: true });
  }, []);

  const stop = useCallback(async () => {
    await invoke("stop_engine");
    setStatus({ running: false });
  }, []);

  return { status, start, stop };
}
