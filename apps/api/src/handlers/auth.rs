// Ubicación: `apps/api/src/handlers/auth.rs`
//
// Descripción: Handlers de autenticación — register, login, refresh, logout.
//
// ADRs relacionados: ADR 0008 (PASETO), ADR 0006 (RBAC)

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use auth::hash_password;
use axum::{
    extract::State,
    response::Json,
    Extension,
};
use domain::{
    entities::User,
    ports::UserRepository,
    value_objects::{Email, PasswordHash},
};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;

/// Request para registro de usuario
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

/// Response de registro exitoso
#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterResponse {
    pub user_id: String,
    pub email: String,
    pub message: String,
}

/// POST /auth/register — Registro de nuevo usuario
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "Auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Usuario registrado exitosamente", body = RegisterResponse),
        (status = 400, description = "Datos inválidos", body = crate::error::ErrorResponse),
        (status = 409, description = "Email ya registrado", body = crate::error::ErrorResponse),
        (status = 500, description = "Error interno", body = crate::error::ErrorResponse),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> ApiResult<Json<RegisterResponse>> {
    info!(email = %body.email, "Register request received");

    // Validar email
    let email = Email::new(&body.email)
        .map_err(|e| ApiError::Validation(format!("Invalid email: {}", e)))?;

    // Verificar si el usuario ya existe
    if let Some(_existing) = state.user_repo.find_active_by_email(&email).await? {
        return Err(ApiError::Conflict(format!(
            "Email '{}' already registered",
            body.email
        )));
    }

    // Hashear password con argon2id
    let password_hash_str = hash_password(&body.password)
        .map_err(|e| ApiError::Internal(format!("Password hashing failed: {}", e)))?;

    // Crear PasswordHash value object
    let password_hash = PasswordHash::new(&password_hash_str)
        .map_err(|e| ApiError::Internal(format!("Invalid password hash: {}", e)))?;

    // Crear usuario
    let user = User::new(email.clone(), password_hash, body.name.clone())
        .map_err(|e| ApiError::Validation(format!("Invalid user data: {:?}", e)))?;

    // Guardar usuario en DB
    state.user_repo.save(&user).await
        .map_err(|e| ApiError::Internal(format!("Failed to save user: {}", e)))?;
    
    info!(user_id = %user.id, "User registered successfully");

    Ok(Json(RegisterResponse {
        user_id: user.id.to_string(),
        email: user.email.value().to_string(),
        message: "User registered successfully".to_string(),
    }))
}

/// Request para login
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Response de login exitoso
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub email_verified_at: Option<String>,
    pub permissions: Vec<String>,
    pub created_at: String,
}

/// POST /auth/login — Autenticación de usuario
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login exitoso", body = LoginResponse),
        (status = 401, description = "Credenciales inválidas", body = crate::error::ErrorResponse),
        (status = 500, description = "Error interno", body = crate::error::ErrorResponse),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    info!(email = %body.email, "Login request received");

    let password_verifier = auth::password::Argon2Verifier;
    
    let use_case = application::auth::login::LoginUseCase::new(
        &state.user_repo,
        &state.session_repo,
        &state.audit_repo,
        &password_verifier,
        &*state.paseto,
    );

    let input = application::auth::login::LoginInput {
        email: body.email.clone(),
        password: body.password,
        ip_address: None, // TODO: Extract from headers
        user_agent: None, // TODO: Extract from headers
    };

    let output = use_case.execute(input).await
        .map_err(|e| match e {
            domain::errors::DomainError::InvalidCredentials => ApiError::Unauthorized("Invalid credentials".to_string()),
            _ => ApiError::Internal(format!("Login error: {}", e)),
        })?;

    info!(user_id = %output.user.id, "User logged in successfully");

    let role = if output.user.email.to_string().contains("admin") {
        "admin"
    } else {
        "user"
    };

    let permissions = state.user_repo.get_permissions(&output.user.id).await
        .map_err(|e| ApiError::Internal(format!("Failed to get permissions: {}", e)))?;

    Ok(Json(LoginResponse {
        access_token: output.access_token,
        refresh_token: output.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 900,
        user: UserResponse {
            id: output.user.id.to_string(),
            email: output.user.email.to_string(),
            name: output.user.name,
            role: role.to_string(),
            is_active: output.user.is_active,
            email_verified_at: output.user.email_verified_at.map(|dt| dt.to_string()),
            permissions,
            created_at: output.user.created_at.to_string(),
        },
    }))
}

/// Request para refresh token
#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Response de refresh exitoso
#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// POST /auth/refresh — Rotación de tokens
#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "Auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Tokens rotados exitosamente", body = RefreshResponse),
        (status = 401, description = "Refresh token inválido", body = crate::error::ErrorResponse),
        (status = 500, description = "Error interno", body = crate::error::ErrorResponse),
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> ApiResult<Json<RefreshResponse>> {
    info!("Refresh token request received");

    let use_case = application::auth::refresh::RefreshUseCase::new(
        &state.session_repo,
        &state.user_repo,
        &*state.paseto,
    );

    let output = use_case.execute(&body.refresh_token).await
        .map_err(|e| match e {
            domain::errors::DomainError::InvalidToken => ApiError::Unauthorized("Invalid refresh token".to_string()),
            domain::errors::DomainError::InvalidCredentials => ApiError::Unauthorized("Invalid refresh token".to_string()),
            _ => ApiError::Internal(format!("Refresh error: {}", e)),
        })?;

    info!("Token rotation completed successfully");

    Ok(Json(RefreshResponse {
        access_token: output.access_token,
        refresh_token: output.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 900, // 15 minutos
    }))
}

/// POST /auth/logout — Cierre de sesión
#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "Auth",
    security(
        ("paseto" = [])
    ),
    responses(
        (status = 200, description = "Logout exitoso"),
        (status = 401, description = "No autenticado", body = crate::error::ErrorResponse),
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::auth::AuthClaims>,
) -> ApiResult<Json<serde_json::Value>> {
    use domain::ports::SessionRepository;
    use tracing::warn;

    let user_id_str = claims.user_id.to_string();

    info!(user_id = %user_id_str, "Logout request received");

    // Revocar TODAS las sesiones del usuario (logout global)
    state.session_repo
        .revoke_all_for_user(&user_id_str)
        .await
        .map_err(|e| {
            warn!(user_id = %user_id_str, error = %e, "Failed to revoke sessions during logout");
            ApiError::Internal(format!("Failed to revoke sessions: {}", e))
        })?;

    info!(
        user_id = %user_id_str,
        "All user sessions revoked successfully"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}
