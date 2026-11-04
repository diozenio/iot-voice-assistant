import { useEffect, useState } from "react";

import ActionButton from "../../../components/ActionButton";
import Panel from "../../../components/Panel";
import type { Settings } from "../../../api/types";
import "./SettingsForm.css";

type SettingsFormProps = {
  settings: Settings | null;
  running: boolean;
  onSave: (settings: Settings) => void;
  onRestart: () => void;
};

function SettingsForm({
  settings,
  running,
  onSave,
  onRestart,
}: SettingsFormProps) {
  const [draft, setDraft] = useState<Settings | null>(settings);
  const [revealKey, setRevealKey] = useState(false);

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  if (!draft) {
    return (
      <Panel title="Configurações">
        <p className="settings-hint">Carregando configurações...</p>
      </Panel>
    );
  }

  const update = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setDraft({ ...draft, [key]: value });

  return (
    <Panel title="Configurações">
      <div className="settings-form">
        <label className="settings-field">
          <span className="settings-label">Porta do WebSocket</span>
          <input
            className="settings-input"
            type="number"
            min={1}
            max={65535}
            value={draft.port}
            onChange={(event) =>
              update("port", Number(event.target.value) || 0)
            }
          />
        </label>

        <label className="settings-field">
          <span className="settings-label">Chave da OpenRouter</span>
          <div className="settings-key">
            <input
              className="settings-input"
              type={revealKey ? "text" : "password"}
              value={draft.openrouterApiKey}
              placeholder="sk-or-v1-..."
              onChange={(event) =>
                update("openrouterApiKey", event.target.value)
              }
            />
            <ActionButton
              label={revealKey ? "Ocultar" : "Mostrar"}
              onClick={() => setRevealKey((value) => !value)}
            />
          </div>
        </label>

        <label className="settings-field">
          <span className="settings-label">Modelo de transcrição</span>
          <input
            className="settings-input"
            value={draft.transcriptionModel}
            placeholder="openai/gpt-4o-transcribe"
            onChange={(event) =>
              update("transcriptionModel", event.target.value)
            }
          />
          <span className="settings-note">
            Precisa ser um modelo de transcrição: <code>openai/gpt-4o-transcribe</code>,{" "}
            <code>openai/gpt-4o-mini-transcribe</code> ou <code>openai/whisper-1</code>.
          </span>
        </label>

        <label className="settings-field">
          <span className="settings-label">Modelo de resposta</span>
          <input
            className="settings-input"
            value={draft.aiModel}
            placeholder="google/gemini-2.5-flash"
            onChange={(event) => update("aiModel", event.target.value)}
          />
        </label>

        <div className="settings-controls">
          <ActionButton
            label="Salvar"
            onClick={() => onSave(draft)}
            variant="primary"
          />
          <ActionButton
            label="Salvar e reiniciar"
            onClick={() => {
              onSave(draft);
              onRestart();
            }}
            disabled={!running}
          />
        </div>

        <p className="settings-hint">
          As configurações ficam salvas em <code>settings.json</code> na pasta
          de configuração do aplicativo e têm prioridade sobre os valores
          padrão do <code>.env</code>.
        </p>
      </div>
    </Panel>
  );
}

export default SettingsForm;
