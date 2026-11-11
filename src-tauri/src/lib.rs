mod actions;
mod audio;
mod commands;
mod config;
mod openrouter;
mod server;
mod state;
mod transcription;
mod tray;
mod window;

use commands::server::{
    clear_logs, get_snapshot, open_recording, open_recordings_folder, restart_server,
    save_settings, start_server, stop_server,
};
use state::{AppState, Level};
use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            start_server,
            stop_server,
            restart_server,
            save_settings,
            clear_logs,
            open_recordings_folder,
            open_recording
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            app.manage(AppState::new(config::load(&handle)));

            tray::setup(app)?;

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Regular);

            window::focus_main(&handle);

            tauri::async_runtime::spawn(async move {
                if let Err(error) = server::start(handle.clone()).await {
                    state::log(
                        &handle,
                        Level::Error,
                        format!("Falha ao iniciar o servidor: {error}"),
                    );
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                window.hide().ok();
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        if let tauri::RunEvent::Reopen { .. } = event {
            window::focus_main(app);
        }

        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        {
            let _ = (app, event);
        }
    });
}
