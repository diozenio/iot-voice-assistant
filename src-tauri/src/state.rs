use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::watch;

use crate::config::Settings;

const MAX_LOGS: usize = 500;
const MAX_CONVERSATION: usize = 200;
const MAX_RECORDINGS: usize = 100;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Info,
    Success,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: u64,
    pub ts: String,
    pub level: Level,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationEntry {
    pub id: u64,
    pub ts: String,
    pub client_id: u64,
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingEntry {
    pub id: u64,
    pub ts: String,
    pub client_id: u64,
    pub file_name: String,
    pub path: String,
    pub duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub id: u64,
    pub address: String,
    pub connected_at: String,
    pub recording: bool,
    pub processing: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
    pub address: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub status: ServerStatus,
    pub settings: Settings,
    pub clients: Vec<ClientInfo>,
    pub logs: Vec<LogEntry>,
    pub conversation: Vec<ConversationEntry>,
    pub recordings: Vec<RecordingEntry>,
}

pub struct ServerHandle {
    pub shutdown: watch::Sender<bool>,
}

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub status: Mutex<ServerStatus>,
    pub server: Mutex<Option<ServerHandle>>,
    pub clients: Mutex<BTreeMap<u64, ClientInfo>>,
    pub logs: Mutex<VecDeque<LogEntry>>,
    pub conversation: Mutex<VecDeque<ConversationEntry>>,
    pub recordings: Mutex<VecDeque<RecordingEntry>>,
    next_id: AtomicU64,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: Mutex::new(settings),
            status: Mutex::new(ServerStatus::default()),
            server: Mutex::new(None),
            clients: Mutex::new(BTreeMap::new()),
            logs: Mutex::new(VecDeque::new()),
            conversation: Mutex::new(VecDeque::new()),
            recordings: Mutex::new(VecDeque::new()),
            next_id: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            status: self.status.lock().unwrap().clone(),
            settings: self.settings.lock().unwrap().clone(),
            clients: self.clients.lock().unwrap().values().cloned().collect(),
            logs: self.logs.lock().unwrap().iter().cloned().collect(),
            conversation: self.conversation.lock().unwrap().iter().cloned().collect(),
            recordings: self.recordings.lock().unwrap().iter().cloned().collect(),
        }
    }
}

fn timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn push<T>(queue: &Mutex<VecDeque<T>>, item: T, max: usize) {
    let mut queue = queue.lock().unwrap();
    queue.push_back(item);
    while queue.len() > max {
        queue.pop_front();
    }
}

pub fn log<R: Runtime>(app: &AppHandle<R>, level: Level, message: impl Into<String>) {
    let state = app.state::<AppState>();
    let entry = LogEntry {
        id: state.next_id(),
        ts: timestamp(),
        level,
        message: message.into(),
    };
    push(&state.logs, entry.clone(), MAX_LOGS);
    let _ = app.emit("log", entry);
}

pub fn conversation<R: Runtime>(
    app: &AppHandle<R>,
    client_id: u64,
    role: &str,
    text: impl Into<String>,
) {
    let state = app.state::<AppState>();
    let entry = ConversationEntry {
        id: state.next_id(),
        ts: timestamp(),
        client_id,
        role: role.to_string(),
        text: text.into(),
    };
    push(&state.conversation, entry.clone(), MAX_CONVERSATION);
    let _ = app.emit("conversation", entry);
}

pub fn recording<R: Runtime>(
    app: &AppHandle<R>,
    client_id: u64,
    file_name: String,
    path: String,
    duration_seconds: f64,
) {
    let state = app.state::<AppState>();
    let entry = RecordingEntry {
        id: state.next_id(),
        ts: timestamp(),
        client_id,
        file_name,
        path,
        duration_seconds,
    };
    push(&state.recordings, entry.clone(), MAX_RECORDINGS);
    let _ = app.emit("recording", entry);
}

fn emit_clients<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let clients: Vec<ClientInfo> = state.clients.lock().unwrap().values().cloned().collect();
    let _ = app.emit("clients", clients);
}

pub fn add_client<R: Runtime>(app: &AppHandle<R>, id: u64, address: String) {
    {
        let state = app.state::<AppState>();
        state.clients.lock().unwrap().insert(
            id,
            ClientInfo {
                id,
                address,
                connected_at: timestamp(),
                recording: false,
                processing: false,
            },
        );
    }
    emit_clients(app);
}

pub fn remove_client<R: Runtime>(app: &AppHandle<R>, id: u64) {
    {
        let state = app.state::<AppState>();
        state.clients.lock().unwrap().remove(&id);
    }
    emit_clients(app);
}

pub fn update_client<R: Runtime>(
    app: &AppHandle<R>,
    id: u64,
    recording: Option<bool>,
    processing: Option<bool>,
) {
    {
        let state = app.state::<AppState>();
        let mut clients = state.clients.lock().unwrap();
        if let Some(client) = clients.get_mut(&id) {
            if let Some(recording) = recording {
                client.recording = recording;
            }
            if let Some(processing) = processing {
                client.processing = processing;
            }
        }
    }
    emit_clients(app);
}

pub fn emit_status<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let status = state.status.lock().unwrap().clone();
    let _ = app.emit("server-status", status);
}

pub fn set_status<R: Runtime>(app: &AppHandle<R>, status: ServerStatus) {
    {
        let state = app.state::<AppState>();
        *state.status.lock().unwrap() = status;
    }
    emit_status(app);
}

pub fn set_stopped<R: Runtime>(app: &AppHandle<R>) {
    {
        let state = app.state::<AppState>();
        state.status.lock().unwrap().running = false;
        state.clients.lock().unwrap().clear();
        *state.server.lock().unwrap() = None;
    }
    emit_status(app);
    emit_clients(app);
}
