import { useState, useEffect } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { Settings, ChevronUp, RefreshCw, Lock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ValidatedInput } from "@/components/ui/validated-input";
import { validatePort } from "@/lib/validators";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import sndwrksLogo from "../../resources/sndwrks-logo.svg";
import constructionDino from "../../resources/sndwrks-construction-dino.svg";
import type {
  Settings as SettingsType,
  OscListenProtocol,
  OscSendProtocol,
  MidiPort,
  EngineStatus,
} from "@/types";

interface SettingsPanelProps {
  settings: SettingsType | null;
  onUpdateSettings: (settings: SettingsType) => void;
  midiInputs: MidiPort[];
  midiOutputs: MidiPort[];
  onRefreshMidi: () => void;
  engineStatus: EngineStatus;
  onStartEngine: () => void;
  onStopEngine: () => void;
}

export function SettingsPanel({
  settings,
  onUpdateSettings,
  midiInputs,
  midiOutputs,
  onRefreshMidi,
  engineStatus,
  onStartEngine,
  onStopEngine,
}: SettingsPanelProps) {
  const [expanded, setExpanded] = useState(false);
  const [version, setVersion] = useState("");

  useEffect(() => {
    getVersion().then(setVersion);
  }, []);

  if (!settings) return null;

  const locked = engineStatus.running;

  const update = (partial: Partial<SettingsType>) => {
    onUpdateSettings({ ...settings, ...partial });
  };

  return (
    <div className="border-b border-border">
      {/* Header - always visible */}
      <div className="flex items-center justify-between py-2">
        <div className="flex items-center gap-3">
          <img src={sndwrksLogo} alt="sndwrks" className="h-8" />
          <span className="text-sm font-semibold tracking-wide">conduit</span>
        </div>
        <div className="flex items-center gap-2">
          {version && (
            <span className="text-xs text-muted-foreground px-2">v{version}</span>
          )}
          <div className="flex items-center gap-1.5">
            <span
              className={`inline-block h-2 w-2 rounded-full ${engineStatus.running ? "bg-green-400" : "bg-red-400"}`}
            />
            <span className="text-xs text-muted-foreground">
              {engineStatus.running ? "Running" : "Stopped"}
            </span>
          </div>
          <Button
            variant={engineStatus.running ? "secondary" : "default"}
            size="sm"
            className="text-xs h-9 w-18"
            onClick={engineStatus.running ? onStopEngine : onStartEngine}
          >
            {engineStatus.running ? "Stop" : "Start"}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-9 w-7 p-0"
            onClick={() => setExpanded(!expanded)}
          >
            {expanded ? (
              <ChevronUp className="h-4 w-4" />
            ) : (
              <Settings className="h-4 w-4" />
            )}
          </Button>
        </div>
      </div>

      {/* Collapsible settings content */}
      {expanded && (
        <div className="pb-3 space-y-3">
          <Separator />
          {locked && (
            <div className="px-4 flex items-center gap-1.5 text-xs text-destructive">
              <Lock className="h-3 w-3" />
              Stop engine to edit settings
            </div>
          )}
          <div className="px-4 grid grid-cols-[1fr_1fr_auto] gap-4">
            {/* OSC Settings */}
            <div className="space-y-2">
              <p className="text-xs font-medium text-secondary-foreground uppercase tracking-wider">
                OSC
              </p>
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="text-xs text-muted-foreground">
                    Listen Port
                  </label>
                  <ValidatedInput
                    inputMode="numeric"
                    className="h-9 text-xs"
                    value={settings.osc_listen_port}
                    disabled={locked}
                    validate={validatePort}
                    errorMessage="Port must be between 1024 and 65535"
                    onCommit={(v) => update({ osc_listen_port: v })}
                  />
                </div>
                <div>
                  <label className="text-xs text-muted-foreground">
                    Listen Protocol
                  </label>
                  <Select
                    value={settings.osc_listen_protocol}
                    disabled={locked}
                    onValueChange={(v) =>
                      update({
                        osc_listen_protocol: v as OscListenProtocol,
                      })
                    }
                  >
                    <SelectTrigger className="h-9 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="udp">UDP</SelectItem>
                      <SelectItem value="tcp">TCP</SelectItem>
                      <SelectItem value="both">Both</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="grid grid-cols-3 gap-2">
                <div className="col-span-1">
                  <label className="text-xs text-muted-foreground">
                    Send IP
                  </label>
                  <Input
                    className="h-9 text-xs"
                    value={settings.osc_send_host}
                    disabled={locked}
                    onChange={(e) =>
                      update({ osc_send_host: e.target.value })
                    }
                  />
                </div>
                <div>
                  <label className="text-xs text-muted-foreground">
                    Send Port
                  </label>
                  <ValidatedInput
                    inputMode="numeric"
                    className="h-9 text-xs"
                    value={settings.osc_send_port}
                    disabled={locked}
                    validate={validatePort}
                    errorMessage="Port must be between 1024 and 65535"
                    onCommit={(v) => update({ osc_send_port: v })}
                  />
                </div>
                <div>
                  <label className="text-xs text-muted-foreground">
                    Send Protocol
                  </label>
                  <Select
                    value={settings.osc_send_protocol}
                    disabled={locked}
                    onValueChange={(v) =>
                      update({
                        osc_send_protocol: v as OscSendProtocol,
                      })
                    }
                  >
                    <SelectTrigger className="h-9 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="udp">UDP</SelectItem>
                      <SelectItem value="tcp">TCP</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </div>

            {/* MIDI Settings */}
            <div className="space-y-2">
              <p className="text-xs font-medium text-secondary-foreground uppercase tracking-wider">
                MIDI
              </p>
              <div>
                <label className="text-xs text-muted-foreground">
                  Input Device
                </label>
                <Select
                  value={settings.midi_input_port_name || ""}
                  disabled={locked}
                  onValueChange={(v) =>
                    update({
                      midi_input_port_name: v || null,
                    })
                  }
                >
                  <SelectTrigger className="h-9 text-xs">
                    <SelectValue placeholder="None" />
                  </SelectTrigger>
                  <SelectContent>
                    {midiInputs.map((p) => (
                      <SelectItem key={p.index} value={p.name}>
                        {p.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div>
                <label className="text-xs text-muted-foreground">
                  Output Device
                </label>
                <Select
                  value={settings.midi_output_port_name || ""}
                  disabled={locked}
                  onValueChange={(v) =>
                    update({
                      midi_output_port_name: v || null,
                    })
                  }
                >
                  <SelectTrigger className="h-9 text-xs">
                    <SelectValue placeholder="None" />
                  </SelectTrigger>
                  <SelectContent>
                    {midiOutputs.map((p) => (
                      <SelectItem key={p.index} value={p.name}>
                        {p.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Refresh MIDI */}
            <div className="flex flex-col items-center gap-2 self-start mt-0.5">
              <Button
                variant="ghost"
                size="sm"
                className="h-6 text-xs gap-1"
                disabled={locked}
                onClick={onRefreshMidi}
              >
                <RefreshCw className="h-3 w-3" />
                Refresh Devices
              </Button>
              <img src={constructionDino} alt="Under construction" className="h-24 w-auto" />
            </div>
          </div>

          <Separator />

          {/* Startup & Engine */}
          <div className="px-4 flex items-center gap-6">
            <label className="flex items-center gap-2 text-xs">
              <Switch
                size="sm"
                checked={settings.launch_on_startup}
                onCheckedChange={(checked) =>
                  update({ launch_on_startup: checked === true })
                }
              />
              <span className="text-muted-foreground">Launch on startup</span>
            </label>
            <label className="flex items-center gap-2 text-xs">
              <Switch
                size="sm"
                checked={settings.engine_auto_start}
                onCheckedChange={(checked) =>
                  update({ engine_auto_start: checked === true })
                }
              />
              <span className="text-muted-foreground">Auto-start engine</span>
            </label>
          </div>
        </div>
      )}
    </div>
  );
}
