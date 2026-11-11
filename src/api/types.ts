export type LogLevel = "info" | "success" | "warn" | "error";

export type LogEntry = {
  id: number;
  ts: string;
  level: LogLevel;
  message: string;
};

export type ConversationRole = "user" | "assistant";

export type ConversationEntry = {
  id: number;
  ts: string;
  role: ConversationRole;
  text: string;
};

export type RecordingEntry = {
  id: number;
  ts: string;
  fileName: string;
  path: string;
  durationSeconds: number;
};

export type DeviceInfo = {
  id: string;
  name: string;
  rssi: number | null;
  connected: boolean;
  lastUsed: boolean;
  recording: boolean;
  processing: boolean;
};

export type BluetoothStatus = {
  available: boolean;
  scanning: boolean;
};

export type Settings = {
  openrouterApiKey: string;
  transcriptionModel: string;
  aiModel: string;
  ttsModel: string;
  ttsVoice: string;
};

export type Snapshot = {
  bluetooth: BluetoothStatus;
  settings: Settings;
  devices: DeviceInfo[];
  logs: LogEntry[];
  conversation: ConversationEntry[];
  recordings: RecordingEntry[];
};
