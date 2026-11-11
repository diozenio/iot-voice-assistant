import { useEffect, useState } from "react";

import { listVoices } from "../../../api/assistant";

import ActionButton from "../../../components/ActionButton";
import Panel from "../../../components/Panel";
import type { Settings } from "../../../api/types";
import "./SettingsForm.css";

type SettingsFormProps = {
  settings: Settings | null;
  onSave: (settings: Settings) => void;
};

function SettingsForm({ settings, onSave }: SettingsFormProps) {
  const [draft, setDraft] = useState<Settings | null>(settings);
  const [revealKey, setRevealKey] = useState(false);
  const [voices, setVoices] = useState<string[]>([]);
  const ttsModel = draft?.ttsModel;

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  useEffect(() => {
    if (!ttsModel) return;
    let active = true;
    const timer = setTimeout(() => {
      listVoices(ttsModel)
        .then((list) => active && setVoices(list))
        .catch(() => active && setVoices([]));
    }, 500);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [ttsModel]);

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
            Precisa ser um modelo de transcrição:{" "}
            <code>openai/gpt-4o-transcribe</code>,{" "}
            <code>openai/gpt-4o-mini-transcribe</code> ou{" "}
            <code>openai/whisper-1</code>.
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

        <label className="settings-field">
          <span className="settings-label">
            Modelo de voz (resposta falada)
          </span>
          <input
            className="settings-input"
            value={draft.ttsModel}
            placeholder="elevenlabs/eleven-flash-v2.5"
            onChange={(event) => update("ttsModel", event.target.value)}
          />
          <span className="settings-note">
            Modelo de síntese de fala da OpenRouter. Veja os disponíveis em{" "}
            <code>openrouter.ai/models?output_modalities=speech</code>.
          </span>
        </label>

        <label className="settings-field">
          <span className="settings-label">Voz</span>
          <select
            className="settings-input"
            value={draft.ttsVoice}
            onChange={(event) => update("ttsVoice", event.target.value)}
          >
            {[...new Set([draft.ttsVoice, ...voices])]
              .filter(Boolean)
              .map((voice) => (
                <option key={voice} value={voice}>
                  {voice}
                </option>
              ))}
          </select>
        </label>

        <div className="settings-controls">
          <ActionButton
            label="Salvar"
            onClick={() => onSave(draft)}
            variant="primary"
          />
        </div>

        <p className="settings-hint">
          As configurações ficam salvas em <code>settings.json</code> na pasta
          de configuração do aplicativo e têm prioridade sobre os valores padrão
          do <code>.env</code>.
        </p>
      </div>
    </Panel>
  );
}

export default SettingsForm;
