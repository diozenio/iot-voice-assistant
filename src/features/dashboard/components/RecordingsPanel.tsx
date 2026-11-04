import ActionButton from "../../../components/ActionButton";
import Panel from "../../../components/Panel";
import type { RecordingEntry } from "../../../api/types";
import { formatDateTime, formatDuration } from "../format";
import "./RecordingsPanel.css";

type RecordingsPanelProps = {
  recordings: RecordingEntry[];
  onOpenFolder: () => void;
  onOpen: (path: string) => void;
};

function RecordingsPanel({
  recordings,
  onOpenFolder,
  onOpen,
}: RecordingsPanelProps) {
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
              <div className="recording-main">
                <span className="recording-name">{recording.fileName}</span>
                <span className="recording-meta">
                  {formatDateTime(recording.ts)} ·{" "}
                  {formatDuration(recording.durationSeconds)}
                </span>
              </div>
              <ActionButton
                label="Abrir"
                onClick={() => onOpen(recording.path)}
              />
            </li>
          ))}
        </ul>
      )}
    </Panel>
  );
}

export default RecordingsPanel;
