import { useEffect, useRef } from "react";

import ActionButton from "../../../components/ActionButton";
import Panel from "../../../components/Panel";
import type { LogEntry } from "../../../api/types";
import { formatTime } from "../format";
import "./LogConsole.css";

type LogConsoleProps = {
  logs: LogEntry[];
  onClear: () => void;
};

function LogConsole({ logs, onClear }: LogConsoleProps) {
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ block: "end" });
  }, [logs.length]);

  return (
    <Panel
      title="Console"
      badge={String(logs.length)}
      actions={<ActionButton label="Limpar" onClick={onClear} />}
    >
      {logs.length === 0 ? (
        <p className="log-empty">Sem registros.</p>
      ) : (
        <div className="log-list">
          {logs.map((entry) => (
            <div key={entry.id} className={`log-line log-${entry.level}`}>
              <span className="log-time">{formatTime(entry.ts)}</span>
              <span className="log-message">{entry.message}</span>
            </div>
          ))}
          <div ref={endRef} />
        </div>
      )}
    </Panel>
  );
}

export default LogConsole;
