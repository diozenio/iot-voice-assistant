import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  BluetoothStatus,
  ConversationEntry,
  DeviceInfo,
  LogEntry,
  RecordingEntry,
  Settings,
  Snapshot,
} from "./types";

export function getSnapshot(): Promise<Snapshot> {
  return invoke("get_snapshot");
}

export function startScan(): Promise<void> {
  return invoke("start_scan");
}

export function stopScan(): Promise<void> {
  return invoke("stop_scan");
}

export function connectDevice(id: string): Promise<void> {
  return invoke("connect_device", { id });
}

export function disconnectDevice(): Promise<void> {
  return invoke("disconnect_device");
}

export function saveSettings(settings: Settings): Promise<void> {
  return invoke("save_settings", { settings });
}

export function clearLogs(): Promise<void> {
  return invoke("clear_logs");
}

export function openRecordingsFolder(): Promise<void> {
  return invoke("open_recordings_folder");
}

export function readRecording(path: string): Promise<ArrayBuffer> {
  return invoke("read_recording", { path });
}

function subscribe<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(event, ({ payload }) => handler(payload));
}

export const onLog = (handler: (entry: LogEntry) => void) =>
  subscribe("log", handler);

export const onConversation = (handler: (entry: ConversationEntry) => void) =>
  subscribe("conversation", handler);

export const onRecording = (handler: (entry: RecordingEntry) => void) =>
  subscribe("recording", handler);

export const onDevices = (handler: (devices: DeviceInfo[]) => void) =>
  subscribe("devices", handler);

export const onBluetoothStatus = (handler: (status: BluetoothStatus) => void) =>
  subscribe("bluetooth-status", handler);
