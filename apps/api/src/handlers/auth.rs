// Ubicación: `apps/api/src/handlers/auth.rs`
//
// Descripción: Handlers de autenticación — register, login, refresh, logout.
//
// ADRs relacionados: ADR 0008 (PASETO), ADR 0006 (RBAC)

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use auth::{hash_password, verify_password, generate_opaque_token, hash_token};
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

    // Validar email
    let email = Email::new(&body.email)
        .map_err(|_| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    // Buscar usuario por email
    let user = state.user_repo.find_active_by_email(&email).await?
        .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    // Verificar password
    let valid = verify_password(&body.password, user.password_hash.as_str())
        .map_err(|e| ApiError::Internal(format!("Password verification error: {}", e)))?;
    if !valid {
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generar access token PASETO v4 (15 minutos)
    let access_token = state.paseto.generate_access_token(&user.id.uuid())
        .map_err(|e| ApiError::Internal(format!("Token generation failed: {}", e)))?;

    // Generar refresh token opaco (32 bytes)
    let raw_refresh = generate_opaque_token();
    let refresh_token_hash = hash_token(&raw_refresh);

    // Crear sesión en DB
    use domain::ports::SessionRepository;
    use time::{Duration, OffsetDateTime};
    let session = domain::Session::new(
        user.id.to_string(),
        refresh_token_hash,
        None, // ip_address - se puede extraer de headers
        None, // user_agent - se puede extraer de headers
        OffsetDateTime::now_utc() + Duration::days(7), // 7 días de expiración
    );

    state.session_repo
        .create(&session)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to create session: {}", e)))?;

    info!(user_id = %user.id, session_id = %session.id, "User logged in successfully");

    let role = if user.email.to_string().contains("admin") {
        "admin"
    } else {
        "user"
    };

    // Obtener permisos del usuario desde la base de datos
    let permissions = state.user_repo.get_permissions(&user.id).await
        .map_err(|e| ApiError::Internal(format!("Failed to get permissions: {}", e)))?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token: raw_refresh,
        token_type: "Bearer".to_string(),
        expires_in: 900,
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email.to_string(),
            name: user.name,
            role: role.to_string(),
            is_active: user.is_active,
            email_verified_at: user.email_verified_at.map(|dt| dt.to_string()),
            permissions,
            created_at: user.created_at.to_string(),
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
    use tracing::warn;
    use domain::ports::SessionRepository;

    info!("Refresh token request received");

    // Hashear el refresh token recibido para buscarlo en DB
    let refresh_hash = hash_token(&body.refresh_token);

    // Buscar sesión por refresh token hash
    let session = state.session_repo
        .find_by_token(&refresh_hash)
        .await
        .map_err(|e| ApiError::Internal(format!("Session lookup failed: {}", e)))?
        .ok_or_else(|| {
            warn!("Refresh token not found in database");
            ApiError::Unauthorized("Invalid refresh token".to_string())
        })?;

    // Verificar que la sesión no esté revocada
    if session.is_revoked {
        warn!(session_id = %session.id, "Attempt to use revoked refresh token");
        return Err(ApiError::Unauthorized("Refresh token has been revoked".to_string()));
    }

    // Verificar que el refresh token no haya expirado
    if session.is_expired() {
        warn!(session_id = %session.id, "Attempt to use expired refresh token");
        return Err(ApiError::Unauthorized("Refresh token has expired".to_string()));
    }

    // Obtener el usuario para generar nuevos tokens
    let user_id = domain::value_objects::UserId::parse(&session.user_id)
        .map_err(|e| ApiError::Internal(format!("Invalid user ID in session: {}", e)))?;
    let user = state
        .user_repo
        .find_by_id(&user_id)
        .await
        .map_err(|e| ApiError::Internal(format!("User lookup failed: {}", e)))?
        .ok_or_else(|| ApiError::Unauthorized("User not found".to_string()))?;

    // Revocar el refresh token anterior (rotación obligatoria)
    state.session_repo
        .revoke(&session.id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to revoke old session: {}", e)))?;

    info!(session_id = %session.id, "Old session revoked due to token rotation");

    // Generar nuevo access token PASETO v4
    let new_access_token = state.paseto
        .generate_access_token(&user.id.uuid())
        .map_err(|e| ApiError::Internal(format!("Token generation failed: {}", e)))?;

    // Generar nuevo refresh token opaco (rotación)
    let new_raw_refresh = generate_opaque_token();
    let new_refresh_hash = hash_token(&new_raw_refresh);

    // Crear nueva sesión con el nuevo refresh token
    use time::{Duration, OffsetDateTime};
    let new_session = domain::Session::new(
        session.user_id.clone(),
        new_refresh_hash,
        session.ip_address.clone(),
        session.user_agent.clone(),
        OffsetDateTime::now_utc() + Duration::days(7), // 7 días
    );

    state.session_repo
        .create(&new_session)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to create new session: {}", e)))?;

    info!(
        user_id = %user.id,
        old_session = %session.id,
        new_session = %new_session.id,
        "Token rotation completed successfully"
    );

    Ok(Json(RefreshResponse {
        access_token: new_access_token,
        refresh_token: new_raw_refresh,
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
