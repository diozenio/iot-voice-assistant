import { useState } from "react";

import ActionButton from "../../../components/ActionButton";
import Panel from "../../../components/Panel";
import type { RecordingEntry } from "../../../api/types";
import { formatDateTime, formatDuration } from "../format";
import RecordingPlayer from "./RecordingPlayer";
import "./RecordingsPanel.css";

type RecordingsPanelProps = {
  recordings: RecordingEntry[];
  onOpenFolder: () => void;
};

function RecordingsPanel({ recordings, onOpenFolder }: RecordingsPanelProps) {
  const [playingId, setPlayingId] = useState<number | null>(null);
  const ordered = [...recordings].reverse();

  return (
    <Panel
      title="Gravações"
      badge={String(recordings.length)}
      actions={<ActionButton label="Abrir pasta" onClick={onOpenFolder} />}
    >
      {ordered.length === 0 ? (
        <p className="recordings-empty">
          Nenhuma gravação nesta sessão. Os arquivos ficam na pasta de dados do
          aplicativo.
        </p>
      ) : (
        <ul className="recordings-list">
          {ordered.map((recording) => (
            <li key={recording.id} className="recording-item">
              <div className="recording-row">
                <div className="recording-main">
                  <span className="recording-name">{recording.fileName}</span>
                  <span className="recording-meta">
                    {formatDateTime(recording.ts)} ·{" "}
                    {formatDuration(recording.durationSeconds)}
                  </span>
                </div>
                <ActionButton
                  label={playingId === recording.id ? "Fechar" : "Ouvir"}
                  onClick={() =>
                    setPlayingId(
                      playingId === recording.id ? null : recording.id,
                    )
                  }
                />
              </div>
              {playingId === recording.id && (
                <RecordingPlayer path={recording.path} />
              )}
            </li>
          ))}
        </ul>
      )}
    </Panel>
  );
}

export default RecordingsPanel;
