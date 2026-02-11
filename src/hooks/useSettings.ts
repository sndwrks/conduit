import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { Settings } from "@/types";

export function useSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then(setSettings)
      .catch((e) => {
        console.error(e);
        toast.error("Failed to load settings");
      })
      .finally(() => setLoading(false));

    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, []);

  const updateSettings = useCallback(
    (updated: Settings) => {
      setSettings(updated);
      if (debounceRef.current) clearTimeout(debounceRef.current);
      debounceRef.current = setTimeout(() => {
        invoke("update_settings", { settings: updated }).catch(console.error);
      }, 500);
    },
    [],
  );

  return { settings, loading, updateSettings };
}
