// Ubicación: `apps/desktop/src-tauri/src/lib.rs`
// 
// Descripción: Aplicación Tauri 2.0 para desktop.
// 
// ADRs relacionados: 0030 (Multiplataforma Tridente)

use tauri::Manager;

mod commands;
mod state;

pub fn run() {
    tracing::info!("Iniciando Boilerplate Desktop...");

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            commands::auth::login,
            commands::auth::logout,
            commands::auth::get_current_user,
            commands::users::list_users,
            commands::users::get_user,
            commands::users::create_user,
        ])
        .setup(|app| {
            tracing::info!("Boilerplate Desktop iniciado correctamente");
            
            // Inicializar base de datos y estado de la app local
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                match state::AppState::new(&app_handle).await {
                    Ok(state) => {
                        app_handle.manage(state);
                        tracing::info!("AppState configurado correctamente");
                    }
                    Err(e) => {
                        tracing::error!("Error inicializando AppState: {}", e);
                        std::process::exit(1);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error al iniciar la aplicación Tauri");
}