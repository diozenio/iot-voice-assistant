import ActionButton from "../../../components/ActionButton";
import type { BluetoothStatus, DeviceInfo } from "../../../api/types";
import "./StatusHeader.css";

type StatusHeaderProps = {
  bluetooth: BluetoothStatus;
  devices: DeviceInfo[];
  onHelp: () => void;
};

function describe(bluetooth: BluetoothStatus, connected?: DeviceInfo): string {
  if (!bluetooth.available) return "Bluetooth indisponível";
  if (connected) return `Conectado a ${connected.name}`;
  if (bluetooth.scanning) return "Procurando o ESP32...";
  return "Nenhum ESP32 conectado";
}

function StatusHeader({ bluetooth, devices, onHelp }: StatusHeaderProps) {
  const connected = devices.find((device) => device.connected);

  return (
    <header className="status-header">
      <div className="status-identity">
        <h1>IoT Voice Assistant</h1>
        <div className="status-line">
          <span
            className={`status-dot ${connected ? "status-dot-online" : "status-dot-offline"}`}
          />
          <span className="status-address">
            {describe(bluetooth, connected)}
          </span>
        </div>
      </div>

      <div className="status-controls">
        <ActionButton label="Ajuda" onClick={onHelp} />
      </div>
    </header>
  );
}

export default StatusHeader;
