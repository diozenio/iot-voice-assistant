//! Link BLE com o ESP32: o computador escaneia, o usuário escolhe o dispositivo no painel
//! e o aplicativo conecta. Protocolo (veja também `firmware/esp/esp.ino`):
//!
//! - `UP` (notify, ESP32 -> PC): `0x01` começou a gravar, `0x02` parou, `0x10` pacote de
//!   áudio ADPCM do microfone (16 kHz mono).
//! - `DOWN` (write sem resposta, PC -> ESP32): `0x01` vai tocar, `0x02` fim do áudio,
//!   `0x10` pacote ADPCM da resposta falada (16 kHz mono), que o ESP32 manda ao fone.

use std::{
    fs,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use btleplug::{
    api::{
        Central, CentralEvent, CharPropFlags, Characteristic, Manager as _, Peripheral as _,
        ScanFilter, WriteType,
    },
    platform::{Adapter, Manager, Peripheral, PeripheralId},
};
use futures_util::StreamExt;
use tauri::{AppHandle, Manager as _, Runtime};
use tokio::{sync::Mutex as AsyncMutex, time::Instant};
use uuid::Uuid;

use crate::{
    actions::{self, Action},
    adpcm::{self, AUDIO_PACKET, SAMPLES_PER_PACKET},
    audio::{MIN_SECONDS, SAMPLE_RATE, duration_seconds, enhance, pcm_to_wav},
    config,
    openrouter::{self, ChatMessage},
    speech,
    state::{self, AppState, Level},
    transcription::build_adapter,
};

const SERVICE: Uuid = Uuid::from_u128(0x7d0a0001_4a7f_4f6c_9c5e_1f2b3c4d5e6f);
const UP: Uuid = Uuid::from_u128(0x7d0a0002_4a7f_4f6c_9c5e_1f2b3c4d5e6f);
const DOWN: Uuid = Uuid::from_u128(0x7d0a0003_4a7f_4f6c_9c5e_1f2b3c4d5e6f);

const EVENT_START: u8 = 0x01;
const EVENT_STOP: u8 = 0x02;
const PLAY_BEGIN: u8 = 0x01;
const PLAY_END: u8 = 0x02;

/// Pacotes enviados de uma vez ao iniciar a fala (~1,3 s): é o colchão do ESP32 contra
/// os intervalos irregulares do BLE. Depois disso o envio segue o ritmo de reprodução.
const PREFILL_PACKETS: usize = 80;
const DEVICE_NAME_PREFIX: &str = "IoT-Voice";

pub struct Bluetooth {
    adapter: Adapter,
    link: AsyncMutex<Option<Link>>,
}

struct Link {
    id: String,
    peripheral: Peripheral,
}

#[derive(Default)]
struct Recorder {
    pcm: Vec<u8>,
    next_seq: Option<u16>,
    lost_packets: usize,
}

struct Session<R: Runtime> {
    app: AppHandle<R>,
    id: String,
    peripheral: Peripheral,
    down: Characteristic,
    http: reqwest::Client,
    recorder: Mutex<Recorder>,
    recording: AtomicBool,
    processing: AtomicBool,
    history: Mutex<Vec<ChatMessage>>,
}

fn scan_filter() -> ScanFilter {
    ScanFilter {
        services: vec![SERVICE],
    }
}

pub async fn init<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let manager = Manager::new().await.map_err(|e| e.to_string())?;
    let adapter = manager
        .adapters()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .next()
        .ok_or("Nenhum adaptador Bluetooth encontrado")?;
    let mut events = adapter.events().await.map_err(|e| e.to_string())?;

    app.manage(Bluetooth {
        adapter: adapter.clone(),
        link: AsyncMutex::new(None),
    });
    state::set_bluetooth(&app, Some(true), None);

    let task_app = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = events.next().await {
            match event {
                CentralEvent::DeviceDiscovered(id) | CentralEvent::DeviceUpdated(id) => {
                    refresh_device(&task_app, &adapter, &id).await;
                }
                CentralEvent::DeviceDisconnected(id) => {
                    lost_link(&task_app, &id.to_string()).await;
                }
                _ => {}
            }
        }
    });

    start_scan(&app).await
}

async fn refresh_device<R: Runtime>(app: &AppHandle<R>, adapter: &Adapter, id: &PeripheralId) {
    let Ok(peripheral) = adapter.peripheral(id).await else {
        return;
    };
    let Ok(Some(properties)) = peripheral.properties().await else {
        return;
    };
    let named = properties
        .local_name
        .as_deref()
        .is_some_and(|name| name.starts_with(DEVICE_NAME_PREFIX));
    if !named && !properties.services.contains(&SERVICE) {
        return;
    }
    state::upsert_device(app, id.to_string(), properties.local_name, properties.rssi);
}

fn bluetooth<R: Runtime>(app: &AppHandle<R>) -> Result<tauri::State<'_, Bluetooth>, String> {
    app.try_state::<Bluetooth>()
        .ok_or_else(|| "Bluetooth indisponível neste computador".to_string())
}

pub async fn start_scan<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let bluetooth = bluetooth(app)?;
    state::forget_idle_devices(app);
    bluetooth
        .adapter
        .start_scan(scan_filter())
        .await
        .map_err(|e| e.to_string())?;
    state::set_bluetooth(app, None, Some(true));
    state::log(app, Level::Info, "Procurando dispositivos Bluetooth...");
    Ok(())
}

pub async fn stop_scan<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let bluetooth = bluetooth(app)?;
    bluetooth
        .adapter
        .stop_scan()
        .await
        .map_err(|e| e.to_string())?;
    state::set_bluetooth(app, None, Some(false));
    Ok(())
}

pub async fn connect<R: Runtime>(app: &AppHandle<R>, id: String) -> Result<(), String> {
    let bluetooth = bluetooth(app)?;
    if bluetooth.link.lock().await.is_some() {
        return Err("Já existe um dispositivo conectado".into());
    }

    let peripheral = bluetooth
        .adapter
        .peripherals()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|peripheral| peripheral.id().to_string() == id)
        .ok_or("Dispositivo não encontrado; busque novamente")?;

    let _ = stop_scan(app).await;
    state::log(app, Level::Info, "Conectando ao ESP32...");
    peripheral.connect().await.map_err(|e| e.to_string())?;

    let opened = match open_session(app, &id, &peripheral).await {
        Ok(session) => session
            .peripheral
            .notifications()
            .await
            .map(|notifications| (session, notifications))
            .map_err(|e| e.to_string()),
        Err(error) => Err(error),
    };
    let (session, mut notifications) = match opened {
        Ok(opened) => opened,
        Err(error) => {
            let _ = peripheral.disconnect().await;
            return Err(error);
        }
    };

    *bluetooth.link.lock().await = Some(Link {
        id: id.clone(),
        peripheral,
    });
    state::set_device_connected(app, &id, true);
    state::log(app, Level::Success, "ESP32 conectado por Bluetooth");

    tauri::async_runtime::spawn(async move {
        while let Some(notification) = notifications.next().await {
            if notification.uuid == UP {
                handle_frame(&session, &notification.value);
            }
        }
        lost_link(&session.app, &session.id).await;
    });
    Ok(())
}

async fn open_session<R: Runtime>(
    app: &AppHandle<R>,
    id: &str,
    peripheral: &Peripheral,
) -> Result<Arc<Session<R>>, String> {
    peripheral
        .discover_services()
        .await
        .map_err(|e| e.to_string())?;
    let characteristics = peripheral.characteristics();
    let find = |uuid: Uuid| {
        characteristics
            .iter()
            .find(|characteristic| characteristic.uuid == uuid)
            .cloned()
    };
    let (Some(up), Some(down)) = (find(UP), find(DOWN)) else {
        return Err("Este dispositivo não tem o firmware do assistente de voz".into());
    };
    if !up.properties.contains(CharPropFlags::NOTIFY) {
        return Err("Firmware incompatível: a característica de áudio não notifica".into());
    }
    peripheral.subscribe(&up).await.map_err(|e| e.to_string())?;

    Ok(Arc::new(Session {
        app: app.clone(),
        id: id.to_string(),
        peripheral: peripheral.clone(),
        down,
        http: reqwest::Client::new(),
        recorder: Mutex::new(Recorder::default()),
        recording: AtomicBool::new(false),
        processing: AtomicBool::new(false),
        history: Mutex::new(vec![ChatMessage::system(actions::SYSTEM_PROMPT)]),
    }))
}

pub async fn disconnect<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let bluetooth = bluetooth(app)?;
    let link = bluetooth.link.lock().await.take();
    let Some(link) = link else {
        return Err("Nenhum dispositivo conectado".into());
    };
    let _ = link.peripheral.disconnect().await;
    finish_link(app, &link.id).await;
    Ok(())
}

/// Chamado quando o Bluetooth cai (ou a notificação termina). Só age se `id` ainda for o
/// dispositivo ativo, então pode ser chamado mais de uma vez com segurança.
async fn lost_link<R: Runtime>(app: &AppHandle<R>, id: &str) {
    let Ok(bluetooth) = bluetooth(app) else {
        return;
    };
    {
        let mut link = bluetooth.link.lock().await;
        if link.as_ref().is_none_or(|link| link.id != id) {
            return;
        }
        *link = None;
    }
    finish_link(app, id).await;
}

async fn finish_link<R: Runtime>(app: &AppHandle<R>, id: &str) {
    state::set_device_connected(app, id, false);
    state::log(app, Level::Warn, "ESP32 desconectado");
    let _ = start_scan(app).await;
}

fn handle_frame<R: Runtime>(session: &Arc<Session<R>>, frame: &[u8]) {
    match frame.first() {
        Some(&EVENT_START) => begin_recording(session),
        Some(&EVENT_STOP) => end_recording(session),
        Some(&AUDIO_PACKET) => store_audio(session, frame),
        _ => {}
    }
}

fn begin_recording<R: Runtime>(session: &Arc<Session<R>>) {
    if session.processing.load(Ordering::Relaxed) {
        state::log(
            &session.app,
            Level::Warn,
            "Ainda processando o comando anterior, ignorando.",
        );
        return;
    }
    *session.recorder.lock().unwrap() = Recorder::default();
    session.recording.store(true, Ordering::Relaxed);
    state::update_device(&session.app, &session.id, Some(true), None);
    state::log(&session.app, Level::Info, "Gravando...");
}

fn end_recording<R: Runtime>(session: &Arc<Session<R>>) {
    if !session.recording.swap(false, Ordering::Relaxed) {
        return;
    }
    state::update_device(&session.app, &session.id, Some(false), None);

    let session = session.clone();
    tauri::async_runtime::spawn(async move {
        finish(session).await;
    });
}

fn store_audio<R: Runtime>(session: &Arc<Session<R>>, frame: &[u8]) {
    if !session.recording.load(Ordering::Relaxed) {
        return;
    }
    let Some((seq, samples)) = adpcm::decode(frame) else {
        return;
    };

    let mut recorder = session.recorder.lock().unwrap();
    if let Some(expected) = recorder.next_seq {
        let missing = seq.wrapping_sub(expected) as usize;
        // Um salto grande é um pacote atrasado ou duplicado, não perda: não preenche.
        if missing > 0 && missing < 500 {
            recorder.lost_packets += missing;
            let silence = vec![0u8; missing * SAMPLES_PER_PACKET * 2];
            recorder.pcm.extend_from_slice(&silence);
        }
    }
    recorder.next_seq = Some(seq.wrapping_add(1));
    for sample in samples {
        recorder.pcm.extend_from_slice(&sample.to_le_bytes());
    }
}

async fn finish<R: Runtime>(session: Arc<Session<R>>) {
    let (pcm, lost) = {
        let mut recorder = session.recorder.lock().unwrap();
        (std::mem::take(&mut recorder.pcm), recorder.lost_packets)
    };
    let duration = duration_seconds(&pcm);

    if lost > 0 {
        state::log(
            &session.app,
            Level::Warn,
            format!("{lost} pacotes de áudio perdidos no Bluetooth"),
        );
    }

    if duration < MIN_SECONDS {
        state::log(
            &session.app,
            Level::Warn,
            format!("Gravação curta demais ({duration:.1}s), ignorada."),
        );
        return;
    }

    session.processing.store(true, Ordering::Relaxed);
    state::update_device(&session.app, &session.id, None, Some(true));

    if let Err(error) = process(&session, &pcm, duration).await {
        state::log(&session.app, Level::Error, format!("Erro: {error}"));
    }

    session.processing.store(false, Ordering::Relaxed);
    state::update_device(&session.app, &session.id, None, Some(false));
}

async fn process<R: Runtime>(
    session: &Arc<Session<R>>,
    pcm: &[u8],
    duration: f64,
) -> Result<(), String> {
    let app = &session.app;
    let settings = app.state::<AppState>().settings.lock().unwrap().clone();

    save_recording(app, pcm, duration)?;

    let adapter = build_adapter(&settings)?;
    state::log(
        app,
        Level::Info,
        format!("Transcrevendo {duration:.1}s de áudio..."),
    );
    let enhanced = enhance(pcm);
    if enhanced.speech_frames == 0 {
        state::log(app, Level::Warn, "Nenhuma fala detectada.");
        return Ok(());
    }
    let text = adapter.transcribe(&enhanced.pcm).await?;

    state::log(app, Level::Info, format!("Transcrição: {text}"));

    if text.is_empty() || text.to_lowercase().contains("[silêncio]") {
        state::log(app, Level::Warn, "Nenhuma fala detectada.");
        return Ok(());
    }

    state::conversation(app, "user", text.clone());

    let history = {
        let mut history = session.history.lock().unwrap();
        history.push(ChatMessage::user(text));
        history.clone()
    };

    let output = match openrouter::complete(
        &session.http,
        &settings.openrouter_api_key,
        &settings.ai_model,
        &history,
    )
    .await
    {
        Ok(reply) => reply,
        Err(error) => {
            session.history.lock().unwrap().pop();
            return Err(error);
        }
    };

    session
        .history
        .lock()
        .unwrap()
        .push(ChatMessage::assistant(output.clone()));

    let decision = actions::parse(&output);
    let reply = decision.reply;

    if decision.action != Action::Reply {
        state::log(
            app,
            Level::Info,
            format!("Ação: {}", decision.action.label()),
        );
        if let Err(error) = actions::execute(app, &decision.action) {
            state::log(
                app,
                Level::Error,
                format!("Falha ao executar a ação: {error}"),
            );
        }
    }

    state::log(app, Level::Success, format!("IA: {reply}"));
    state::conversation(app, "assistant", reply.clone());

    if let Err(error) = speak(session, &settings, &reply).await {
        state::log(
            app,
            Level::Error,
            format!("Falha ao falar a resposta: {error}"),
        );
    }

    Ok(())
}

async fn write<R: Runtime>(session: &Session<R>, data: &[u8]) -> Result<(), String> {
    session
        .peripheral
        .write(&session.down, data, WriteType::WithoutResponse)
        .await
        .map_err(|e| e.to_string())
}

/// Sintetiza a resposta e a envia ao ESP32 em tempo real, sem estourar o buffer dele.
async fn speak<R: Runtime>(
    session: &Session<R>,
    settings: &config::Settings,
    text: &str,
) -> Result<(), String> {
    let samples = speech::synthesize(
        &session.http,
        &settings.openrouter_api_key,
        &settings.tts_model,
        &settings.tts_voice,
        text,
    )
    .await?;

    let mut encoder = adpcm::Encoder::new();
    let packets: Vec<Vec<u8>> = samples
        .chunks(SAMPLES_PER_PACKET)
        .map(|block| encoder.encode(block))
        .collect();

    let interval = Duration::from_secs_f64(SAMPLES_PER_PACKET as f64 / SAMPLE_RATE as f64);
    write(session, &[PLAY_BEGIN]).await?;

    let started = Instant::now();
    for (index, packet) in packets.iter().enumerate() {
        if index >= PREFILL_PACKETS {
            tokio::time::sleep_until(started + interval * (index - PREFILL_PACKETS) as u32).await;
        }
        write(session, packet).await?;
    }

    write(session, &[PLAY_END]).await?;
    state::log(
        &session.app,
        Level::Info,
        format!(
            "Resposta enviada ao fone ({:.1}s)",
            duration_seconds_i16(&samples)
        ),
    );
    Ok(())
}

fn duration_seconds_i16(samples: &[i16]) -> f64 {
    samples.len() as f64 / SAMPLE_RATE as f64
}

fn save_recording<R: Runtime>(app: &AppHandle<R>, pcm: &[u8], duration: f64) -> Result<(), String> {
    let dir = config::recordings_dir(app)?;
    let file_name = format!(
        "{}.wav",
        chrono::Utc::now()
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
            .replace([':', '.'], "-")
    );
    let path = dir.join(&file_name);

    fs::write(&path, pcm_to_wav(pcm)).map_err(|e| e.to_string())?;

    state::log(
        app,
        Level::Info,
        format!("Áudio salvo em {}", path.display()),
    );
    state::recording(app, file_name, path.to_string_lossy().to_string(), duration);

    Ok(())
}
