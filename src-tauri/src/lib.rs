mod managers;
mod shortcut;

use log::info;
use managers::audio::AudioRecordingManager;
use managers::transcription::TranscriptionManager;
use rdev::{simulate, EventType, Key, SimulateError};
use std::sync::Arc;
use std::{thread, time};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutEvent, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_stronghold::Builder::new(|_pass| todo!()).build())
        .plugin(tauri_plugin_upload::init())
        // .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--auto-launch"]),
        ))
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let _tray = TrayIconBuilder::new().build(app)?;

            let vad_path = app.path().resolve(
                "resources/silero_vad_v4.onnx",
                tauri::path::BaseDirectory::Resource,
            )?;

            let whisper_path = app.path().resolve(
                "resources/ggml-small.bin",
                tauri::path::BaseDirectory::Resource,
            )?;

            let recording_manager = Arc::new(
                AudioRecordingManager::new(&vad_path)
                    .expect("Failed to initialize recording manager"),
            );
            let transcription_manager = Arc::new(
                TranscriptionManager::new(
                    whisper_path
                        .to_str()
                        .expect("Path contains invalid UTF-8 Chars"),
                )
                .expect("Failed to initialize transcription manager"),
            );

            // Add managers to Tauri's managed state
            app.manage(recording_manager.clone());
            app.manage(transcription_manager.clone());

            shortcut::enable_shortcut(app);

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();

                // TODO may be different on windows, this works for macos
                tauri::AppHandle::hide(window.app_handle()).unwrap();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
