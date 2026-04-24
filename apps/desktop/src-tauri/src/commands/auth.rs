// Ubicación: `apps/desktop/src-tauri/src/commands/auth.rs`
// 
// Descripción: Comandos de autenticación para Tauri.
// 
// ADRs relacionados: 0030, 0008

#[tauri::command]
pub async fn login(
    email: String,
    _password: String,
) -> Result<serde_json::Value, String> {
    tracing::info!("Login attempt for: {}", email);
    Err("No implementado".to_string())
}

#[tauri::command]
pub async fn logout() -> Result<(), String> {
    tracing::info!("Logout");
    Ok(())
}

#[tauri::command]
pub async fn get_current_user() -> Result<Option<serde_json::Value>, String> {
    Ok(None)
}