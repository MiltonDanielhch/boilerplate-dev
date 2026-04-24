// Ubicación: `apps/desktop/src-tauri/src/commands/users.rs`
// 
// Descripción: Comandos de gestión de usuarios para Tauri.
// 
// ADRs relacionados: 0030, 0006

#[tauri::command]
pub async fn list_users(
    _page: Option<i64>,
    _search: Option<String>,
) -> Result<serde_json::Value, String> {
    tracing::info!("List users");
    Ok(serde_json::json!({
        "users": [],
        "total": 0
    }))
}

#[tauri::command]
pub async fn get_user(id: String) -> Result<serde_json::Value, String> {
    tracing::info!("Get user: {}", id);
    Err("No implementado".to_string())
}