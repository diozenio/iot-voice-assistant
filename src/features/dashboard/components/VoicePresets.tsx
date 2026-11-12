import { useEffect, useRef, useState } from "react";

import { listVoicePresets, previewVoice } from "../../../api/assistant";
import ActionButton from "../../../components/ActionButton";
import type { Settings, VoicePreset } from "../../../api/types";
import "./VoicePresets.css";

type VoicePresetsProps = {
  draft: Settings;
  onSelect: (preset: VoicePreset) => void;
};

function isSelected(draft: Settings, preset: VoicePreset): boolean {
  return (
    draft.ttsModel === preset.model &&
    draft.ttsVoice === preset.voice &&
    draft.ttsInstructions === preset.instructions
  );
}

function VoicePresets({ draft, onSelect }: VoicePresetsProps) {
  const [presets, setPresets] = useState<VoicePreset[]>([]);
  const [loadingId, setLoadingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const audio = useRef<HTMLAudioElement | null>(null);

  useEffect(() => {
    listVoicePresets()
      .then(setPresets)
      .catch(() => setPresets([]));
    return () => {
      audio.current?.pause();
    };
  }, []);

  const preview = async (preset: VoicePreset) => {
    setError(null);
    setLoadingId(preset.id);
    try {
      const bytes = await previewVoice(
        preset.model,
        preset.voice,
        preset.instructions,
      );
      audio.current?.pause();
      const url = URL.createObjectURL(new Blob([bytes], { type: "audio/wav" }));
      const player = new Audio(url);
      player.onended = () => URL.revokeObjectURL(url);
      audio.current = player;
      await player.play();
    } catch (cause) {
      setError(typeof cause === "string" ? cause : String(cause));
    } finally {
      setLoadingId(null);
    }
  };

  if (presets.length === 0) return null;

  return (
    <div className="voice-presets">
      <span className="settings-label">Vozes prontas</span>
      <ul className="voice-presets-list">
        {presets.map((preset) => {
          const selected = isSelected(draft, preset);
          return (
            <li
              key={preset.id}
              className={`voice-preset ${selected ? "voice-preset-selected" : ""}`}
            >
              <button
                type="button"
                className="voice-preset-main"
                onClick={() => onSelect(preset)}
                aria-pressed={selected}
              >
                <span className="voice-preset-title">
                  {preset.name}
                  <span className="voice-preset-emotion">{preset.emotion}</span>
                </span>
                <span className="voice-preset-description">
                  {preset.description}
                </span>
                <span className="voice-preset-provider">{preset.provider}</span>
              </button>
              <ActionButton
                label={loadingId === preset.id ? "..." : "Ouvir"}
                onClick={() => preview(preset)}
                disabled={loadingId !== null}
              />
            </li>
          );
        })}
      </ul>
      {error && <span className="voice-presets-error">{error}</span>}
      <span className="settings-note">
        Escolha uma voz e clique em Salvar. O botão Ouvir toca uma amostra no
        computador (usa a chave da OpenRouter).
      </span>
    </div>
  );
}

export default VoicePresets;
