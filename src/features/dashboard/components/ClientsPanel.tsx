import Panel from "../../../components/Panel";
import type { ClientInfo } from "../../../api/types";
import { formatTime } from "../format";
import "./ClientsPanel.css";

type ClientsPanelProps = {
  clients: ClientInfo[];
};

function stateLabel(client: ClientInfo): { label: string; kind: string } {
  if (client.processing) return { label: "Processando", kind: "processing" };
  if (client.recording) return { label: "Gravando", kind: "recording" };
  return { label: "Ocioso", kind: "idle" };
}

function ClientsPanel({ clients }: ClientsPanelProps) {
  return (
    <Panel title="Conexões" badge={String(clients.length)}>
      {clients.length === 0 ? (
        <p className="clients-empty">Nenhum ESP32 conectado.</p>
      ) : (
        <ul className="clients-list">
          {clients.map((client) => {
            const state = stateLabel(client);
            return (
              <li key={client.id} className="client-item">
                <div className="client-main">
                  <span className="client-address">{client.address}</span>
                  <span className="client-since">
                    desde {formatTime(client.connectedAt)}
                  </span>
                </div>
                <span className={`client-state client-state-${state.kind}`}>
                  {state.label}
                </span>
              </li>
            );
          })}
        </ul>
      )}
    </Panel>
  );
}

export default ClientsPanel;
