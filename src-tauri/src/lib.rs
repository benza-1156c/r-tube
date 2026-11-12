#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::Builder;

pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod services;
pub mod state;
pub mod tray;
pub mod utils;

use state::AppState;

include!(concat!(env!("OUT_DIR"), "/auto_commands.rs"));

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(app_commands!());

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                window.hide().unwrap();
            }
            _ => {}
        })
        .setup(move |app| {
            builder.mount_events(app);

            #[cfg(desktop)]
            tray::create_tray(app).expect("Failed to initialize system tray");

            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let app_path = app_handle
                    .path()
                    .app_data_dir()
                    .expect("Failed to get app_data_dir");

                let db = db::init_db(&app_path)
                    .await
                    .expect("Failed to initialize database");

                let yt = std::sync::Arc::new(
                    services::youtube::YoutubeClient::new()
                        .await
                        .expect("Failed to initialize YouTube client"),
                );

                let hls_yt = std::sync::Arc::clone(&yt);
                utils::proxy::set_manifest_refresher(move |video_id| {
                    let yt = std::sync::Arc::clone(&hls_yt);
                    Box::pin(async move { yt.hls_manifest_url(&video_id).await })
                });

                let audio_yt = std::sync::Arc::clone(&yt);
                utils::proxy::set_audio_refresher(move |video_id, itag| {
                    let yt = std::sync::Arc::clone(&audio_yt);
                    Box::pin(async move { yt.refresh_audio_url(&video_id, itag).await })
                });

                let _ = yt.init_from_active_profile(&db).await;

                // Build the InnerTube clients off the UI thread: they fetch and
                // parse YouTube's player script, which used to stall startup.
                let warm = std::sync::Arc::clone(&yt);
                tauri::async_runtime::spawn(async move {
                    warm.warm_up().await;
                });

                let initial_settings = models::settings::AppSettings::load_from_store(&app_handle);
                let settings = std::sync::Arc::new(tokio::sync::RwLock::new(initial_settings));

                app_handle.manage(AppState::new(db, app_path, yt, settings));
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
