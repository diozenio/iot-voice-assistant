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
  clientId: number;
  role: ConversationRole;
  text: string;
};

export type RecordingEntry = {
  id: number;
  ts: string;
  clientId: number;
  fileName: string;
  path: string;
  durationSeconds: number;
};

export type ClientInfo = {
  id: number;
  address: string;
  connectedAt: string;
  recording: boolean;
  processing: boolean;
};

export type ServerStatus = {
  running: boolean;
  port: number;
  address: string;
};

export type Settings = {
  port: number;
  openrouterApiKey: string;
  transcriptionModel: string;
  aiModel: string;
};

export type Snapshot = {
  status: ServerStatus;
  settings: Settings;
  clients: ClientInfo[];
  logs: LogEntry[];
  conversation: ConversationEntry[];
  recordings: RecordingEntry[];
};
