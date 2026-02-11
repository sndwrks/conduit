import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import type { Mapping } from "@/types";
import { defaultMapping } from "@/types";

export function useMappings() {
  const [mappings, setMappings] = useState<Mapping[]>([]);
  const [loading, setLoading] = useState(true);
  const debounceTimers = useRef<Map<string, ReturnType<typeof setTimeout>>>(
    new Map(),
  );

  useEffect(() => {
    invoke<Mapping[]>("get_mappings")
      .then(setMappings)
      .catch((e) => {
        console.error(e);
        toast.error("Failed to load mappings");
      })
      .finally(() => setLoading(false));

    const timers = debounceTimers.current;
    return () => {
      timers.forEach((t) => clearTimeout(t));
    };
  }, []);

  const addMapping = useCallback(async () => {
    const mapping = defaultMapping();
    const id = await invoke<string>("add_mapping", { mapping });
    const updated = [...(await invoke<Mapping[]>("get_mappings"))];
    setMappings(updated);
    return id;
  }, []);

  const updateMapping = useCallback((mapping: Mapping) => {
    setMappings((prev) => prev.map((m) => (m.id === mapping.id ? mapping : m)));

    const existing = debounceTimers.current.get(mapping.id);
    if (existing) clearTimeout(existing);

    debounceTimers.current.set(
      mapping.id,
      setTimeout(() => {
        invoke("update_mapping", { mapping }).catch((e) => {
          console.error(e);
          toast.error("Failed to save mapping");
        });
        debounceTimers.current.delete(mapping.id);
      }, 300),
    );
  }, []);

  const deleteMapping = useCallback(
    async (id: string) => {
      await invoke("delete_mapping", { id });
      setMappings((prev) => prev.filter((m) => m.id !== id));
    },
    [],
  );

  const reorderMappings = useCallback(
    async (ids: string[]) => {
      await invoke("reorder_mappings", { ids });
      setMappings((prev) => {
        const map = new Map(prev.map((m) => [m.id, m]));
        return ids.map((id) => map.get(id)!).filter(Boolean);
      });
    },
    [],
  );

  return { mappings, loading, addMapping, updateMapping, deleteMapping, reorderMappings };
}
