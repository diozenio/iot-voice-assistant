import { useCallback, useEffect, useState } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";

import * as api from "../../api/assistant";
import type {
  BluetoothStatus,
  ConversationEntry,
  DeviceInfo,
  LogEntry,
  RecordingEntry,
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

export function useAssistant() {
  const [bluetooth, setBluetooth] = useState<BluetoothStatus>({
    available: false,
    scanning: false,
  });
  const [settings, setSettings] = useState<Settings | null>(null);
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
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
        setBluetooth(snapshot.bluetooth);
        setSettings(snapshot.settings);
        setDevices(snapshot.devices);
        setLogs(snapshot.logs);
        setConversation(snapshot.conversation);
        setRecordings(snapshot.recordings);
      })
      .catch((cause) => active && setError(toMessage(cause)));

    Promise.all([
      api.onBluetoothStatus(setBluetooth),
      api.onDevices(setDevices),
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

  const startScan = useCallback(() => run(api.startScan), [run]);
  const stopScan = useCallback(() => run(api.stopScan), [run]);
  const connect = useCallback(
    (id: string) => run(() => api.connectDevice(id)),
    [run],
  );
  const disconnect = useCallback(() => run(api.disconnectDevice), [run]);
  const openRecordingsFolder = useCallback(
    () => run(api.openRecordingsFolder),
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
    bluetooth,
    settings,
    devices,
    logs,
    conversation,
    recordings,
    error,
    dismissError: () => setError(null),
    startScan,
    stopScan,
    connect,
    disconnect,
    save,
    clearLogs,
    openRecordingsFolder,
  };
}
