use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::Builder;

pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod services;
pub mod state;
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
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

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

                let _ = yt.init_from_active_profile(&db).await;

                app_handle.manage(AppState::new(db, app_path, yt));
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
