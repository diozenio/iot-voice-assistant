import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  ClientInfo,
  ConversationEntry,
  LogEntry,
  RecordingEntry,
  ServerStatus,
  Settings,
  Snapshot,
} from "./types";

export function getSnapshot(): Promise<Snapshot> {
  return invoke("get_snapshot");
}

export function startServer(): Promise<void> {
  return invoke("start_server");
}

export function stopServer(): Promise<void> {
  return invoke("stop_server");
}

export function restartServer(): Promise<void> {
  return invoke("restart_server");
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

export function openRecording(path: string): Promise<void> {
  return invoke("open_recording", { path });
}

function subscribe<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, ({ payload }) => handler(payload));
}

export const onLog = (handler: (entry: LogEntry) => void) => subscribe("log", handler);

export const onConversation = (handler: (entry: ConversationEntry) => void) =>
  subscribe("conversation", handler);

export const onRecording = (handler: (entry: RecordingEntry) => void) =>
  subscribe("recording", handler);

export const onClients = (handler: (clients: ClientInfo[]) => void) =>
  subscribe("clients", handler);

export const onServerStatus = (handler: (status: ServerStatus) => void) =>
  subscribe("server-status", handler);
