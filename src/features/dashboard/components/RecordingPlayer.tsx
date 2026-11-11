import { useEffect, useState } from "react";

import { readRecording } from "../../../api/assistant";

type RecordingPlayerProps = {
  path: string;
};

function RecordingPlayer({ path }: RecordingPlayerProps) {
  const [url, setUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let objectUrl: string | null = null;
    let cancelled = false;

    readRecording(path)
      .then((bytes) => {
        if (cancelled) return;
        objectUrl = URL.createObjectURL(
          new Blob([bytes], { type: "audio/wav" }),
        );
        setUrl(objectUrl);
      })
      .catch((cause) => {
        if (!cancelled) setError(String(cause));
      });

    return () => {
      cancelled = true;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [path]);

  if (error) return <span className="recording-error">{error}</span>;
  if (!url) return <span className="recording-meta">Carregando...</span>;

  return <audio className="recording-audio" src={url} controls autoPlay />;
}

export default RecordingPlayer;
