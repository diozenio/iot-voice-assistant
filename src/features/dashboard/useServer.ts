import { useCallback, useEffect, useState } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";

import * as api from "../../api/server";
import type {
  ClientInfo,
  ConversationEntry,
  LogEntry,
  RecordingEntry,
  ServerStatus,
  Settings,
} from "../../api/types";

const MAX_LOGS = 500;
const MAX_CONVERSATION = 200;
const MAX_RECORDINGS = 100;

function append<T>(list: T[], item: T, max: number): T[] {
  const next = [...list, item];
  return next.length > max ? next.slice(next.length - max) : next;
}

function toMessage(error: unknown): string {
  return typeof error === "string" ? error : String(error);
}

export function useServer() {
  const [status, setStatus] = useState<ServerStatus>({
    running: false,
    port: 0,
    address: "",
  });
  const [settings, setSettings] = useState<Settings | null>(null);
  const [clients, setClients] = useState<ClientInfo[]>([]);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [conversation, setConversation] = useState<ConversationEntry[]>([]);
  const [recordings, setRecordings] = useState<RecordingEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    const unlisteners: UnlistenFn[] = [];

    api
      .getSnapshot()
      .then((snapshot) => {
        if (!active) return;
        setStatus(snapshot.status);
        setSettings(snapshot.settings);
        setClients(snapshot.clients);
        setLogs(snapshot.logs);
        setConversation(snapshot.conversation);
        setRecordings(snapshot.recordings);
      })
      .catch((cause) => active && setError(toMessage(cause)));

    Promise.all([
      api.onServerStatus(setStatus),
      api.onClients(setClients),
      api.onLog((entry) => setLogs((list) => append(list, entry, MAX_LOGS))),
      api.onConversation((entry) =>
        setConversation((list) => append(list, entry, MAX_CONVERSATION)),
      ),
      api.onRecording((entry) =>
        setRecordings((list) => append(list, entry, MAX_RECORDINGS)),
      ),
    ])
      .then((fns) => {
        if (active) unlisteners.push(...fns);
        else fns.forEach((fn) => fn());
      })
      .catch((cause) => active && setError(toMessage(cause)));

    return () => {
      active = false;
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  const run = useCallback(async (action: () => Promise<void>) => {
    setError(null);
    try {
      await action();
    } catch (cause) {
      setError(toMessage(cause));
    }
  }, []);

  const start = useCallback(() => run(api.startServer), [run]);
  const stop = useCallback(() => run(api.stopServer), [run]);
  const restart = useCallback(() => run(api.restartServer), [run]);
  const openRecordingsFolder = useCallback(
    () => run(api.openRecordingsFolder),
    [run],
  );
  const openRecording = useCallback(
    (path: string) => run(() => api.openRecording(path)),
    [run],
  );

  const clearLogs = useCallback(
    () =>
      run(async () => {
        await api.clearLogs();
        setLogs([]);
      }),
    [run],
  );

  const save = useCallback(
    (next: Settings) =>
      run(async () => {
        await api.saveSettings(next);
        setSettings(next);
      }),
    [run],
  );

  return {
    status,
    settings,
    clients,
    logs,
    conversation,
    recordings,
    error,
    dismissError: () => setError(null),
    start,
    stop,
    restart,
    save,
    clearLogs,
    openRecordingsFolder,
    openRecording,
  };
}
