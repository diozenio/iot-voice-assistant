import { useEffect, useRef } from "react";

import Panel from "../../../components/Panel";
import type { ConversationEntry } from "../../../api/types";
import { formatTime } from "../format";
import "./ConversationPanel.css";

type ConversationPanelProps = {
  conversation: ConversationEntry[];
};

function ConversationPanel({ conversation }: ConversationPanelProps) {
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ block: "end" });
  }, [conversation.length]);

  return (
    <Panel title="Conversa">
      {conversation.length === 0 ? (
        <p className="conversation-empty">
          Aguardando comandos de voz do ESP32.
        </p>
      ) : (
        <ul className="conversation-list">
          {conversation.map((entry) => (
            <li
              key={entry.id}
              className={`message message-${entry.role}`}
            >
              <div className="message-meta">
                <span className="message-role">
                  {entry.role === "user" ? "Transcrição" : "IA"}
                </span>
                <span className="message-time">{formatTime(entry.ts)}</span>
              </div>
              <p className="message-text">{entry.text}</p>
            </li>
          ))}
          <div ref={endRef} />
        </ul>
      )}
    </Panel>
  );
}

export default ConversationPanel;
