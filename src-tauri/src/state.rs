use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::config::Settings;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

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
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingEntry {
    pub id: u64,
    pub ts: String,
    pub file_name: String,
    pub path: String,
    pub duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub rssi: Option<i16>,
    pub connected: bool,
    pub recording: bool,
    pub processing: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BluetoothStatus {
    pub available: bool,
    pub scanning: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub bluetooth: BluetoothStatus,
    pub settings: Settings,
    pub devices: Vec<DeviceInfo>,
    pub logs: Vec<LogEntry>,
    pub conversation: Vec<ConversationEntry>,
    pub recordings: Vec<RecordingEntry>,
}

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub bluetooth: Mutex<BluetoothStatus>,
    pub devices: Mutex<BTreeMap<String, DeviceInfo>>,
    pub logs: Mutex<VecDeque<LogEntry>>,
    pub conversation: Mutex<VecDeque<ConversationEntry>>,
    pub recordings: Mutex<VecDeque<RecordingEntry>>,
    next_id: AtomicU64,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: Mutex::new(settings),
            bluetooth: Mutex::new(BluetoothStatus::default()),
            devices: Mutex::new(BTreeMap::new()),
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
            bluetooth: self.bluetooth.lock().unwrap().clone(),
            settings: self.settings.lock().unwrap().clone(),
            devices: self.devices.lock().unwrap().values().cloned().collect(),
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

pub fn conversation<R: Runtime>(app: &AppHandle<R>, role: &str, text: impl Into<String>) {
    let state = app.state::<AppState>();
    let entry = ConversationEntry {
        id: state.next_id(),
        ts: timestamp(),
        role: role.to_string(),
        text: text.into(),
    };
    push(&state.conversation, entry.clone(), MAX_CONVERSATION);
    let _ = app.emit("conversation", entry);
}

pub fn recording<R: Runtime>(
    app: &AppHandle<R>,
    file_name: String,
    path: String,
    duration_seconds: f64,
) {
    let state = app.state::<AppState>();
    let entry = RecordingEntry {
        id: state.next_id(),
        ts: timestamp(),
        file_name,
        path,
        duration_seconds,
    };
    push(&state.recordings, entry.clone(), MAX_RECORDINGS);
    let _ = app.emit("recording", entry);
}

fn emit_devices<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let devices: Vec<DeviceInfo> = state.devices.lock().unwrap().values().cloned().collect();
    let _ = app.emit("devices", devices);
}

pub fn upsert_device<R: Runtime>(
    app: &AppHandle<R>,
    id: String,
    name: Option<String>,
    rssi: Option<i16>,
) {
    {
        let state = app.state::<AppState>();
        let mut devices = state.devices.lock().unwrap();
        let device = devices.entry(id.clone()).or_insert_with(|| DeviceInfo {
            name: id.clone(),
            id,
            rssi: None,
            connected: false,
            recording: false,
            processing: false,
        });
        if let Some(name) = name {
            device.name = name;
        }
        if rssi.is_some() {
            device.rssi = rssi;
        }
    }
    emit_devices(app);
}

pub fn set_device_connected<R: Runtime>(app: &AppHandle<R>, id: &str, connected: bool) {
    {
        let state = app.state::<AppState>();
        if let Some(device) = state.devices.lock().unwrap().get_mut(id) {
            device.connected = connected;
            if !connected {
                device.recording = false;
                device.processing = false;
            }
        }
    }
    emit_devices(app);
}

pub fn update_device<R: Runtime>(
    app: &AppHandle<R>,
    id: &str,
    recording: Option<bool>,
    processing: Option<bool>,
) {
    {
        let state = app.state::<AppState>();
        if let Some(device) = state.devices.lock().unwrap().get_mut(id) {
            if let Some(recording) = recording {
                device.recording = recording;
            }
            if let Some(processing) = processing {
                device.processing = processing;
            }
        }
    }
    emit_devices(app);
}

pub fn forget_idle_devices<R: Runtime>(app: &AppHandle<R>) {
    {
        let state = app.state::<AppState>();
        state
            .devices
            .lock()
            .unwrap()
            .retain(|_, device| device.connected);
    }
    emit_devices(app);
}

pub fn set_bluetooth<R: Runtime>(
    app: &AppHandle<R>,
    available: Option<bool>,
    scanning: Option<bool>,
) {
    let status = {
        let state = app.state::<AppState>();
        let mut status = state.bluetooth.lock().unwrap();
        if let Some(available) = available {
            status.available = available;
        }
        if let Some(scanning) = scanning {
            status.scanning = scanning;
        }
        status.clone()
    };
    let _ = app.emit("bluetooth-status", status);
}
