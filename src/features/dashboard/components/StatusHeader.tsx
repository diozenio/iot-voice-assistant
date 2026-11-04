import ActionButton from "../../../components/ActionButton";
import type { ServerStatus } from "../../../api/types";
import "./StatusHeader.css";

type StatusHeaderProps = {
  status: ServerStatus;
  clientCount: number;
  onStart: () => void;
  onStop: () => void;
  onRestart: () => void;
};

function StatusHeader({
  status,
  clientCount,
  onStart,
  onStop,
  onRestart,
}: StatusHeaderProps) {
  return (
    <header className="status-header">
      <div className="status-identity">
        <h1>IoT Voice Assistant</h1>
        <div className="status-line">
          <span
            className={`status-dot ${status.running ? "status-dot-online" : "status-dot-offline"}`}
          />
          <span className="status-address">
            {status.running ? status.address : "Servidor parado"}
          </span>
          {status.running && (
            <span className="status-clients">
              {clientCount === 1
                ? "1 dispositivo conectado"
                : `${clientCount} dispositivos conectados`}
            </span>
          )}
        </div>
      </div>

      <div className="status-controls">
        {status.running ? (
          <>
            <ActionButton label="Reiniciar" onClick={onRestart} />
            <ActionButton label="Parar" onClick={onStop} variant="danger" />
          </>
        ) : (
          <ActionButton label="Iniciar servidor" onClick={onStart} variant="primary" />
        )}
      </div>
    </header>
  );
}

export default StatusHeader;
