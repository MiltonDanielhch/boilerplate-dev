// Ubicación: `apps/desktop/src-tauri/src/commands/auth.rs`
// 
// Descripción: Comandos de autenticación para Tauri.
// 
// ADRs relacionados: 0030, 0008

use application::auth::login::{LoginInput, LoginUseCase};
use auth::Argon2Verifier;
use domain::ports::UserRepository;
use tauri::State;
use tauri_plugin_store::StoreExt;

#[tauri::command]
pub async fn login(
    email: String,
    password: String,
    state: State<'_, crate::state::AppState>,
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    tracing::info!("Login attempt for: {}", email);

    let password_verifier = Argon2Verifier;

    let use_case = LoginUseCase::new(
        &*state.user_repo,
        &*state.session_repo,
        &*state.audit_repo,
        &password_verifier,
        &*state.paseto,
    );

    let input = LoginInput {
        email,
        password,
        ip_address: Some("localhost".to_string()),
        user_agent: Some("Tauri Desktop".to_string()),
    };

    let output = use_case.execute(input).await
        .map_err(|e| format!("Error en login: {}", e))?;

    // Guardar tokens en el store local
    if let Ok(store) = app_handle.store("auth.bin") {
        store.set("access_token", serde_json::json!(output.access_token.clone()));
        store.set("refresh_token", serde_json::json!(output.refresh_token.clone()));
        store.set("user_id", serde_json::json!(output.user.id.to_string()));
        let _ = store.save();
    }

    Ok(serde_json::json!({
        "user": {
            "id": output.user.id.to_string(),
            "email": output.user.email.to_string(),
            "name": output.user.name,
            "is_active": output.user.is_active,
        },
        "access_token": output.access_token,
        "refresh_token": output.refresh_token,
    }))
}

#[tauri::command]
pub async fn logout(app_handle: tauri::AppHandle) -> Result<(), String> {
    tracing::info!("Logout");
    
    // Limpiar tokens
    if let Ok(store) = app_handle.store("auth.bin") {
        store.delete("access_token");
        store.delete("refresh_token");
        let _ = store.save();
    }
    
    Ok(())
}

#[tauri::command]
pub async fn get_current_user(app_handle: tauri::AppHandle, state: State<'_, crate::state::AppState>) -> Result<Option<serde_json::Value>, String> {
    if let Ok(store) = app_handle.store("auth.bin") {
        if let Some(token_val) = store.get("access_token") {
            if let Some(token_str) = token_val.as_str() {
                if let Ok(claims) = state.paseto.verify(token_str) {
                    if let Ok(user_id) = domain::value_objects::UserId::parse(&claims.sub) {
                        if let Ok(Some(user)) = state.user_repo.find_by_id(&user_id).await {
                            // Obtener permisos
                            let permissions = state.user_repo.get_permissions(&user_id).await
                                .unwrap_or_default();
                            
                            return Ok(Some(serde_json::json!({
                                "id": user.id.to_string(),
                                "email": user.email.to_string(),
                                "name": user.name,
                                "is_active": user.is_active,
                                "permissions": permissions,
                            })));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}