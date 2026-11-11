use std::{
    fs,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tauri::{AppHandle, Manager, Runtime};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex as AsyncMutex, watch},
};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};

use crate::{
    actions::{self, Action},
    audio::{MIN_SECONDS, duration_seconds, pcm_to_wav},
    config::{self, Settings},
    openrouter::{self, ChatMessage},
    state::{self, AppState, Level, ServerHandle, ServerStatus},
    transcription::{TranscriptionAdapter, build_adapter},
};

type Sink = Arc<AsyncMutex<SplitSink<WebSocketStream<TcpStream>, Message>>>;

struct Connection {
    chunks: Mutex<Vec<u8>>,
    recording: AtomicBool,
    processing: AtomicBool,
    history: Mutex<Vec<ChatMessage>>,
}

impl Connection {
    fn new() -> Self {
        Self {
            chunks: Mutex::new(Vec::new()),
            recording: AtomicBool::new(false),
            processing: AtomicBool::new(false),
            history: Mutex::new(vec![ChatMessage::system(actions::SYSTEM_PROMPT)]),
        }
    }
}

struct Context<R: Runtime> {
    app: AppHandle<R>,
    client_id: u64,
    settings: Settings,
    adapter: Arc<dyn TranscriptionAdapter>,
    http: reqwest::Client,
}

fn local_ip() -> String {
    local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "localhost".to_string())
}

pub async fn start<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let settings = {
        let state = app.state::<AppState>();
        if state.server.lock().unwrap().is_some() {
            return Err("O servidor já está em execução".into());
        }
        state.settings.lock().unwrap().clone()
    };

    let adapter = build_adapter(&settings)?;

    let listener = TcpListener::bind(("0.0.0.0", settings.port))
        .await
        .map_err(|e| format!("Não foi possível abrir a porta {}: {e}", settings.port))?;

    let address = format!("ws://{}:{}", local_ip(), settings.port);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    {
        let state = app.state::<AppState>();
        *state.server.lock().unwrap() = Some(ServerHandle {
            shutdown: shutdown_tx,
        });
    }

    state::set_status(
        &app,
        ServerStatus {
            running: true,
            port: settings.port,
            address: address.clone(),
        },
    );
    state::log(
        &app,
        Level::Success,
        format!("Servidor WebSocket ouvindo em {address}"),
    );
    state::log(
        &app,
        Level::Info,
        format!(
            "Transcrição: {} · IA: {}",
            settings.transcription_model, settings.ai_model
        ),
    );

    let task_app = app.clone();
    tauri::async_runtime::spawn(async move {
        accept_loop(task_app, listener, shutdown_rx, settings, adapter).await;
    });

    Ok(())
}

pub fn stop<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let handle = {
        let state = app.state::<AppState>();
        state.server.lock().unwrap().take()
    };
    let Some(handle) = handle else {
        return Err("O servidor não está em execução".into());
    };
    let _ = handle.shutdown.send(true);
    state::log(app, Level::Info, "Servidor WebSocket encerrado");
    Ok(())
}

async fn accept_loop<R: Runtime>(
    app: AppHandle<R>,
    listener: TcpListener,
    mut shutdown: watch::Receiver<bool>,
    settings: Settings,
    adapter: Arc<dyn TranscriptionAdapter>,
) {
    let mut next_client_id = 0u64;

    loop {
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
            accepted = listener.accept() => match accepted {
                Ok((stream, addr)) => {
                    next_client_id += 1;
                    let context = Arc::new(Context {
                        app: app.clone(),
                        client_id: next_client_id,
                        settings: settings.clone(),
                        adapter: adapter.clone(),
                        http: reqwest::Client::new(),
                    });
                    let shutdown = shutdown.clone();
                    tauri::async_runtime::spawn(async move {
                        let app = context.app.clone();
                        let id = context.client_id;
                        if let Err(error) = handle_connection(context, addr, stream, shutdown).await {
                            state::log(&app, Level::Error, format!("Erro no WebSocket: {error}"));
                        }
                        state::remove_client(&app, id);
                    });
                }
                Err(error) => {
                    state::log(&app, Level::Error, format!("Erro ao aceitar conexão: {error}"));
                }
            }
        }
    }

    state::set_stopped(&app);
}

async fn handle_connection<R: Runtime>(
    ctx: Arc<Context<R>>,
    addr: SocketAddr,
    stream: TcpStream,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), String> {
    let websocket = accept_async(stream).await.map_err(|e| e.to_string())?;

    state::add_client(&ctx.app, ctx.client_id, addr.to_string());
    state::log(
        &ctx.app,
        Level::Success,
        format!("ESP32 conectado ({addr})"),
    );

    let (sink, reader) = websocket.split();
    let sink: Sink = Arc::new(AsyncMutex::new(sink));
    let connection = Arc::new(Connection::new());

    let result = message_loop(&ctx, &connection, &sink, reader, &mut shutdown).await;

    state::log(&ctx.app, Level::Info, "ESP32 desconectado");
    result
}

async fn message_loop<R: Runtime>(
    ctx: &Arc<Context<R>>,
    connection: &Arc<Connection>,
    sink: &Sink,
    mut reader: SplitStream<WebSocketStream<TcpStream>>,
    shutdown: &mut watch::Receiver<bool>,
) -> Result<(), String> {
    loop {
        let incoming = tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    let _ = sink.lock().await.close().await;
                    return Ok(());
                }
                continue;
            }
            incoming = reader.next() => incoming,
        };

        let Some(incoming) = incoming else {
            return Ok(());
        };

        match incoming.map_err(|e| e.to_string())? {
            Message::Binary(data) => {
                if connection.recording.load(Ordering::Relaxed) {
                    connection.chunks.lock().unwrap().extend_from_slice(&data);
                }
            }
            Message::Text(text) => handle_command(ctx, connection, sink, text.as_str()),
            Message::Close(_) => return Ok(()),
            _ => {}
        }
    }
}

fn handle_command<R: Runtime>(
    ctx: &Arc<Context<R>>,
    connection: &Arc<Connection>,
    sink: &Sink,
    command: &str,
) {
    match command {
        "start" => {
            if connection.processing.load(Ordering::Relaxed) {
                state::log(
                    &ctx.app,
                    Level::Warn,
                    "Ainda processando o comando anterior, ignorando.",
                );
                return;
            }
            connection.chunks.lock().unwrap().clear();
            connection.recording.store(true, Ordering::Relaxed);
            state::update_client(&ctx.app, ctx.client_id, Some(true), None);
            state::log(&ctx.app, Level::Info, "Gravando...");
        }
        "stop" if connection.recording.load(Ordering::Relaxed) => {
            connection.recording.store(false, Ordering::Relaxed);
            state::update_client(&ctx.app, ctx.client_id, Some(false), None);

            let ctx = ctx.clone();
            let connection = connection.clone();
            let sink = sink.clone();
            tauri::async_runtime::spawn(async move {
                finish(ctx, connection, sink).await;
            });
        }
        _ => {}
    }
}

async fn finish<R: Runtime>(ctx: Arc<Context<R>>, connection: Arc<Connection>, sink: Sink) {
    let pcm = std::mem::take(&mut *connection.chunks.lock().unwrap());
    let duration = duration_seconds(&pcm);

    if duration < MIN_SECONDS {
        state::log(
            &ctx.app,
            Level::Warn,
            format!("Gravação curta demais ({duration:.1}s), ignorada."),
        );
        return;
    }

    connection.processing.store(true, Ordering::Relaxed);
    state::update_client(&ctx.app, ctx.client_id, None, Some(true));

    if let Err(error) = process(&ctx, &connection, &sink, &pcm, duration).await {
        state::log(&ctx.app, Level::Error, format!("Erro: {error}"));
    }

    connection.processing.store(false, Ordering::Relaxed);
    state::update_client(&ctx.app, ctx.client_id, None, Some(false));
}

async fn process<R: Runtime>(
    ctx: &Arc<Context<R>>,
    connection: &Arc<Connection>,
    sink: &Sink,
    pcm: &[u8],
    duration: f64,
) -> Result<(), String> {
    save_recording(ctx, pcm, duration)?;

    state::log(
        &ctx.app,
        Level::Info,
        format!("Transcrevendo {duration:.1}s de áudio..."),
    );
    let text = ctx.adapter.transcribe(pcm).await?;

    state::log(&ctx.app, Level::Info, format!("Transcrição: {text}"));

    if text.is_empty() || text.to_lowercase().contains("[silêncio]") {
        state::log(&ctx.app, Level::Warn, "Nenhuma fala detectada.");
        return Ok(());
    }

    state::conversation(&ctx.app, ctx.client_id, "user", text.clone());

    let history = {
        let mut history = connection.history.lock().unwrap();
        history.push(ChatMessage::user(text));
        history.clone()
    };

    let output = match openrouter::complete(
        &ctx.http,
        &ctx.settings.openrouter_api_key,
        &ctx.settings.ai_model,
        &history,
    )
    .await
    {
        Ok(reply) => reply,
        Err(error) => {
            connection.history.lock().unwrap().pop();
            return Err(error);
        }
    };

    connection
        .history
        .lock()
        .unwrap()
        .push(ChatMessage::assistant(output.clone()));

    let decision = actions::parse(&output);
    let reply = decision.reply;

    if decision.action != Action::Reply {
        state::log(
            &ctx.app,
            Level::Info,
            format!("Ação: {}", decision.action.label()),
        );
        if let Err(error) = actions::execute(&ctx.app, &decision.action) {
            state::log(
                &ctx.app,
                Level::Error,
                format!("Falha ao executar a ação: {error}"),
            );
        }
    }

    state::log(&ctx.app, Level::Success, format!("IA: {reply}"));
    state::conversation(&ctx.app, ctx.client_id, "assistant", reply.clone());

    let _ = sink.lock().await.send(Message::text(reply)).await;

    Ok(())
}

fn save_recording<R: Runtime>(
    ctx: &Arc<Context<R>>,
    pcm: &[u8],
    duration: f64,
) -> Result<(), String> {
    let dir = config::recordings_dir(&ctx.app)?;
    let file_name = format!(
        "{}.wav",
        chrono::Utc::now()
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
            .replace([':', '.'], "-")
    );
    let path = dir.join(&file_name);

    fs::write(&path, pcm_to_wav(pcm)).map_err(|e| e.to_string())?;

    state::log(
        &ctx.app,
        Level::Info,
        format!("Áudio salvo em {}", path.display()),
    );
    state::recording(
        &ctx.app,
        ctx.client_id,
        file_name,
        path.to_string_lossy().to_string(),
        duration,
    );

    Ok(())
}
