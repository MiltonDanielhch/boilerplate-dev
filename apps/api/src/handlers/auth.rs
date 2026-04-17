// Ubicación: `apps/api/src/handlers/auth.rs`
//
// Descripción: Handlers de autenticación — register, login, refresh, logout.
//
// ADRs relacionados: ADR 0008 (PASETO), ADR 0006 (RBAC)

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use auth::{hash_password, verify_password};
use axum::{
    extract::State,
    response::Json,
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
    if let Some(existing) = state.user_repo.find_active_by_email(&email).await? {
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
    let raw_refresh = auth::generate_opaque_token();
    let refresh_token = auth::hash_token(&raw_refresh);

    // TODO: Guardar refresh token en DB (token_repo)
    // TODO: Crear sesión (session_repo)
    // TODO: Registrar audit log

    info!(user_id = %user.id, "User logged in successfully");

    Ok(Json(LoginResponse {
        access_token,
        refresh_token: raw_refresh, // Devolver el token raw al cliente
        token_type: "Bearer".to_string(),
        expires_in: 900, // 15 minutos en segundos
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

    // TODO: Verificar refresh token hash en DB
    // TODO: Revocar refresh token anterior (rotación obligatoria)
    // TODO: Generar nuevos access_token + refresh_token
    // TODO: Guardar nuevo refresh token

    // Placeholder implementation
    Err(ApiError::Internal(
        "Refresh token not yet implemented".to_string()
    ))
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
) -> ApiResult<Json<serde_json::Value>> {
    info!("Logout request received");

    // TODO: Extraer token del header Authorization
    // TODO: Revocar sesión
    // TODO: Revocar refresh token
    // TODO: Registrar audit log

    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}
