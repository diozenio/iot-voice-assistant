import { useState } from "react";

import Tabs, { type Tab } from "../../components/Tabs";
import ConversationPanel from "./components/ConversationPanel";
import DevicesPanel from "./components/DevicesPanel";
import HelpModal from "./components/HelpModal";
import LogConsole from "./components/LogConsole";
import RecordingsPanel from "./components/RecordingsPanel";
import SettingsForm from "./components/SettingsForm";
import StatusHeader from "./components/StatusHeader";
import { useAssistant } from "./useAssistant";
import "./Dashboard.css";

type TabId = "monitor" | "recordings" | "settings";

const TABS: Tab<TabId>[] = [
  { id: "monitor", label: "Monitor" },
  { id: "recordings", label: "Gravações" },
  { id: "settings", label: "Configurações" },
];

function Dashboard() {
  const [tab, setTab] = useState<TabId>("monitor");
  const [helpOpen, setHelpOpen] = useState(false);
  const assistant = useAssistant();

  return (
    <main className="dashboard">
      <StatusHeader
        bluetooth={assistant.bluetooth}
        devices={assistant.devices}
        onHelp={() => setHelpOpen(true)}
      />

      {assistant.error && (
        <div className="dashboard-error" role="alert">
          <span>{assistant.error}</span>
          <button
            className="dashboard-error-close"
            onClick={assistant.dismissError}
            aria-label="Fechar aviso"
          >
            ×
          </button>
        </div>
      )}

      <Tabs tabs={TABS} active={tab} onChange={setTab} />

      {tab === "monitor" && (
        <div className="dashboard-monitor">
          <div className="dashboard-column">
            <ConversationPanel conversation={assistant.conversation} />
          </div>
          <div className="dashboard-column">
            <DevicesPanel
              bluetooth={assistant.bluetooth}
              devices={assistant.devices}
              onScan={assistant.startScan}
              onStopScan={assistant.stopScan}
              onConnect={assistant.connect}
              onDisconnect={assistant.disconnect}
            />
            <LogConsole logs={assistant.logs} onClear={assistant.clearLogs} />
          </div>
        </div>
      )}

      {tab === "recordings" && (
        <div className="dashboard-stack">
          <RecordingsPanel
            recordings={assistant.recordings}
            onOpenFolder={assistant.openRecordingsFolder}
          />
        </div>
      )}

      {tab === "settings" && (
        <div className="dashboard-stack">
          <SettingsForm settings={assistant.settings} onSave={assistant.save} />
        </div>
      )}
      {helpOpen && <HelpModal onClose={() => setHelpOpen(false)} />}
    </main>
  );
}

export default Dashboard;
