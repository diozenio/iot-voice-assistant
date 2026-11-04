import { useState } from "react";

import Tabs, { type Tab } from "../../components/Tabs";
import ClientsPanel from "./components/ClientsPanel";
import ConversationPanel from "./components/ConversationPanel";
import LogConsole from "./components/LogConsole";
import RecordingsPanel from "./components/RecordingsPanel";
import SettingsForm from "./components/SettingsForm";
import StatusHeader from "./components/StatusHeader";
import DashboardActions from "./DashboardActions";
import { useServer } from "./useServer";
import "./Dashboard.css";

type TabId = "monitor" | "recordings" | "settings";

const TABS: Tab<TabId>[] = [
  { id: "monitor", label: "Monitor" },
  { id: "recordings", label: "Gravações" },
  { id: "settings", label: "Configurações" },
];

function Dashboard() {
  const [tab, setTab] = useState<TabId>("monitor");
  const server = useServer();

  return (
    <main className="dashboard">
      <StatusHeader
        status={server.status}
        clientCount={server.clients.length}
        onStart={server.start}
        onStop={server.stop}
        onRestart={server.restart}
      />

      {server.error && (
        <div className="dashboard-error" role="alert">
          <span>{server.error}</span>
          <button
            className="dashboard-error-close"
            onClick={server.dismissError}
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
            <ConversationPanel conversation={server.conversation} />
          </div>
          <div className="dashboard-column">
            <ClientsPanel clients={server.clients} />
            <LogConsole logs={server.logs} onClear={server.clearLogs} />
          </div>
        </div>
      )}

      {tab === "recordings" && (
        <div className="dashboard-stack">
          <RecordingsPanel
            recordings={server.recordings}
            onOpenFolder={server.openRecordingsFolder}
            onOpen={server.openRecording}
          />
        </div>
      )}

      {tab === "settings" && (
        <div className="dashboard-stack">
          <SettingsForm
            settings={server.settings}
            running={server.status.running}
            onSave={server.save}
            onRestart={server.restart}
          />
          <DashboardActions />
        </div>
      )}
    </main>
  );
}

export default Dashboard;
