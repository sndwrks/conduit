import { useEffect, useRef } from "react";
import { Toaster, toast } from "sonner";
import { validatePort } from "@/lib/validators";
import { SettingsPanel } from "@/components/SettingsPanel";
import { MappingTable } from "@/components/MappingTable";
import { ActivityLog } from "@/components/ActivityLog";
import { useSettings } from "@/hooks/useSettings";
import { useMappings } from "@/hooks/useMappings";
import { useMidi } from "@/hooks/useMidi";
import { useEngine } from "@/hooks/useEngine";
import { useActivityLog } from "@/hooks/useActivityLog";

function App() {
  const { settings, updateSettings } = useSettings();
  const { mappings, addMapping, updateMapping, deleteMapping } = useMappings();
  const { inputs, outputs, refresh: refreshMidi } = useMidi();
  const { status, start, stop } = useEngine();
  const { entries, paused, clear, togglePause } = useActivityLog();

  const autoStarted = useRef(false);

  useEffect(() => {
    if (settings?.engine_auto_start && !autoStarted.current) {
      autoStarted.current = true;
      handleStart();
    }
  }, [settings]);

  const handleStart = async () => {
    if (settings) {
      if (!validatePort(String(settings.osc_listen_port)).valid) {
        toast.error("OSC Listen Port must be between 1024 and 65535");
        return;
      }
      if (!validatePort(String(settings.osc_send_port)).valid) {
        toast.error("OSC Send Port must be between 1024 and 65535");
        return;
      }
    }
    try {
      await start();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "An unexpected error occurred");
    }
  };

  const handleStop = async () => {
    try {
      await stop();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "An unexpected error occurred");
    }
  };

  const handleAddMapping = async () => {
    try {
      await addMapping();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "An unexpected error occurred");
    }
  };

  const handleDeleteMapping = async (id: string) => {
    try {
      await deleteMapping(id);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "An unexpected error occurred");
    }
  };

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <SettingsPanel
        settings={settings}
        onUpdateSettings={updateSettings}
        midiInputs={inputs}
        midiOutputs={outputs}
        onRefreshMidi={refreshMidi}
        engineStatus={status}
        onStartEngine={handleStart}
        onStopEngine={handleStop}
      />
      <MappingTable
        mappings={mappings}
        onUpdateMapping={updateMapping}
        onDeleteMapping={handleDeleteMapping}
        onAddMapping={handleAddMapping}
      />
      <ActivityLog
        entries={entries}
        paused={paused}
        onTogglePause={togglePause}
        onClear={clear}
      />
      <Toaster theme="dark" position="bottom-right" />
    </div>
  );
}

export default App;
