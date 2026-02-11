import { useState, useRef, useEffect } from "react";
import { ChevronDown, ChevronUp, Pause, Play, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { LogEntry } from "@/components/LogEntry";
import type { LogEntry as LogEntryType } from "@/hooks/useActivityLog";

interface ActivityLogProps {
  entries: LogEntryType[];
  paused: boolean;
  onTogglePause: () => void;
  onClear: () => void;
}

export function ActivityLog({
  entries,
  paused,
  onTogglePause,
  onClear,
}: ActivityLogProps) {
  const [expanded, setExpanded] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current && !paused) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [entries, paused]);

  const maxHeight = expanded ? "max-h-64" : "max-h-20";

  return (
    <div className="border-t border-border">
      <div className="flex items-center justify-between px-3 py-1">
        <span className="text-xs text-muted-foreground">
          Activity Log ({entries.length})
        </span>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0"
            onClick={onTogglePause}
            title={paused ? "Resume" : "Pause"}
          >
            {paused ? (
              <Play className="h-3 w-3" />
            ) : (
              <Pause className="h-3 w-3" />
            )}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0"
            onClick={onClear}
            title="Clear"
          >
            <X className="h-3 w-3" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0"
            onClick={() => setExpanded(!expanded)}
          >
            {expanded ? (
              <ChevronDown className="h-3 w-3" />
            ) : (
              <ChevronUp className="h-3 w-3" />
            )}
          </Button>
        </div>
      </div>
      <div
        ref={scrollRef}
        className={`overflow-y-auto px-3 pb-1 ${maxHeight} transition-all`}
      >
        {entries.length === 0 ? (
          <p className="text-xs text-muted-foreground/50 py-1">
            No activity yet
          </p>
        ) : (
          entries.map((entry) => <LogEntry key={entry.id} entry={entry} />)
        )}
      </div>
    </div>
  );
}
