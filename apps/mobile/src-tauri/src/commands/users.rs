// Ubicación: `apps/desktop/src-tauri/src/commands/users.rs`
// 
// Descripción: Comandos de gestión de usuarios para Tauri.
// 
// ADRs relacionados: 0030, 0006

use application::auth::register::{RegisterInput, RegisterUseCase};
use auth::Argon2Verifier;
use domain::ports::UserRepository;
use tauri::State;

#[tauri::command]
pub async fn list_users(
    page: Option<i64>,
    _search: Option<String>,
    state: State<'_, crate::state::AppState>,
) -> Result<serde_json::Value, String> {
    tracing::info!("List users page: {:?}", page);
    
    // Default limit=10, page starts at 1
    let limit = 10;
    let page_num = page.unwrap_or(1).max(1);
    let offset = (page_num - 1) * limit;

    let users = state.user_repo.list(limit, offset).await
        .map_err(|e| format!("Error listing users: {}", e))?;

    let users_json: Vec<serde_json::Value> = users.into_iter().map(|u| {
        serde_json::json!({
            "id": u.id.to_string(),
            "email": u.email.to_string(),
            "name": u.name,
            "is_active": u.is_active,
            "created_at": u.created_at.to_string(),
        })
    }).collect();

    Ok(serde_json::json!({
        "users": users_json,
    }))
}

#[tauri::command]
pub async fn get_user(
    id: String,
    state: State<'_, crate::state::AppState>,
) -> Result<serde_json::Value, String> {
    tracing::info!("Get user: {}", id);
    
    let user_id = domain::value_objects::UserId::parse(&id)
        .map_err(|_| "Invalid User ID".to_string())?;

    let user = state.user_repo.find_by_id(&user_id).await
        .map_err(|e| format!("Error getting user: {}", e))?
        .ok_or_else(|| "User not found".to_string())?;

    Ok(serde_json::json!({
        "id": user.id.to_string(),
        "email": user.email.to_string(),
        "name": user.name,
        "is_active": user.is_active,
        "created_at": user.created_at.to_string(),
    }))
}

#[tauri::command]
pub async fn create_user(
    email: String,
    password: String,
    name: Option<String>,
    state: State<'_, crate::state::AppState>,
) -> Result<serde_json::Value, String> {
    tracing::info!("Create user: {}", email);

    // NoopMailer para escritorio (por ahora)
    struct NoopMailer;
    impl domain::ports::Mailer for NoopMailer {
        fn send(&self, _message: &domain::ports::EmailMessage) -> impl std::future::Future<Output = Result<(), domain::errors::DomainError>> + Send {
            async { Ok(()) }
        }
        fn send_verification_email(&self, _to: &str, _name: &str, _url: &str) -> impl std::future::Future<Output = Result<(), domain::errors::DomainError>> + Send {
            async { Ok(()) }
        }
        fn send_password_reset(&self, _to: &str, _url: &str) -> impl std::future::Future<Output = Result<(), domain::errors::DomainError>> + Send {
            async { Ok(()) }
        }
    }
    let mailer = NoopMailer;
    let password_hasher = Argon2Verifier;

    let use_case = RegisterUseCase::new(
        &*state.user_repo,
        &mailer,
        &password_hasher,
        &*state.audit_repo,
    );

    let input = RegisterInput {
        email,
        password,
        name,
    };

    let user = use_case.execute(input).await
        .map_err(|e| format!("Error creating user: {}", e))?;

    Ok(serde_json::json!({
        "id": user.id.to_string(),
        "email": user.email.to_string(),
        "name": user.name,
        "created_at": user.created_at.to_string(),
    }))
}