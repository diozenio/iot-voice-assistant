import ActionButton from "../../../components/ActionButton";
import Panel from "../../../components/Panel";
import type { BluetoothStatus, DeviceInfo } from "../../../api/types";
import "./DevicesPanel.css";

type DevicesPanelProps = {
  bluetooth: BluetoothStatus;
  devices: DeviceInfo[];
  onScan: () => void;
  onStopScan: () => void;
  onConnect: (id: string) => void;
  onDisconnect: () => void;
};

function stateLabel(device: DeviceInfo): { label: string; kind: string } {
  if (device.processing) return { label: "Processando", kind: "processing" };
  if (device.recording) return { label: "Gravando", kind: "recording" };
  if (device.connected) return { label: "Conectado", kind: "connected" };
  return { label: "Disponível", kind: "idle" };
}

function DevicesPanel({
  bluetooth,
  devices,
  onScan,
  onStopScan,
  onConnect,
  onDisconnect,
}: DevicesPanelProps) {
  const busy = devices.some((device) => device.connected);

  return (
    <Panel
      title="Bluetooth"
      badge={String(devices.length)}
      actions={
        <ActionButton
          label={bluetooth.scanning ? "Parar busca" : "Buscar"}
          onClick={bluetooth.scanning ? onStopScan : onScan}
          disabled={!bluetooth.available}
        />
      }
    >
      {!bluetooth.available ? (
        <p className="devices-empty">
          Bluetooth indisponível. Ligue o Bluetooth do computador e permita o
          acesso ao aplicativo.
        </p>
      ) : devices.length === 0 ? (
        <p className="devices-empty">
          {bluetooth.scanning
            ? "Procurando o ESP32... ligue-o e deixe por perto."
            : "Nenhum ESP32 encontrado. Clique em Buscar."}
        </p>
      ) : (
        <ul className="devices-list">
          {devices.map((device) => {
            const state = stateLabel(device);
            return (
              <li key={device.id} className="device-item">
                <div className="device-main">
                  <span className="device-name">{device.name}</span>
                  <span className="device-meta">
                    {device.rssi === null ? "sem sinal" : `${device.rssi} dBm`}
                  </span>
                </div>
                <span className={`device-state device-state-${state.kind}`}>
                  {state.label}
                </span>
                {device.connected ? (
                  <ActionButton
                    label="Desconectar"
                    onClick={onDisconnect}
                    variant="danger"
                  />
                ) : (
                  <ActionButton
                    label="Conectar"
                    onClick={() => onConnect(device.id)}
                    variant="primary"
                    disabled={busy}
                  />
                )}
              </li>
            );
          })}
        </ul>
      )}
    </Panel>
  );
}

export default DevicesPanel;
