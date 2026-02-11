import type { LogEntry as LogEntryType } from "@/hooks/useActivityLog";

interface LogEntryProps {
  entry: LogEntryType;
}

export function LogEntry({ entry }: LogEntryProps) {
  const time = entry.timestamp.split("T").pop()?.split(".")[0] || entry.timestamp;

  if (entry.type === "unmatched") {
    return (
      <div className="text-xs font-mono text-muted-foreground leading-tight">
        <span className="text-muted-foreground/60">{time}</span>{" "}
        <span>
          ? {entry.protocol?.toUpperCase()} {entry.display}
        </span>
      </div>
    );
  }

  const inputColor =
    entry.inputProtocol === "osc" ? "text-blue-400" : "text-green-400";
  const outputColor =
    entry.outputProtocol === "osc" ? "text-blue-400" : "text-green-400";

  return (
    <div className="text-xs font-mono leading-tight">
      <span className="text-muted-foreground/60">{time}</span>{" "}
      <span className={inputColor}>
        {entry.inputProtocol?.toUpperCase()} {entry.inputDisplay}
      </span>{" "}
      <span className="text-muted-foreground">→</span>{" "}
      <span className={outputColor}>
        {entry.outputProtocol?.toUpperCase()} {entry.outputDisplay}
      </span>
    </div>
  );
}
