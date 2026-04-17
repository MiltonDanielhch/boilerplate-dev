// Ubicación: `apps/api/src/error.rs`
//
// Descripción: Manejo centralizado de errores HTTP.
//              Convierte errores de dominio a respuestas HTTP.
//
// ADRs relacionados: ADR 0007 (Errores), ADR 0003 (Axum)

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use domain::errors::DomainError;
use serde::Serialize;
use tracing::error;
use utoipa::ToSchema;

/// Respuesta de error estandarizada.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

/// Detalle del error.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Error de la API que puede convertirse en respuesta HTTP.
#[derive(Debug)]
pub enum ApiError {
    Domain(DomainError),
    Validation(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ApiError::Domain(e) => map_domain_error(e),
            ApiError::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone())
            }
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            ApiError::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone())
            }
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            ApiError::Internal(msg) => {
                error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".to_string(), // No exponer detalles internos
                )
            }
        };

        let body = Json(ErrorResponse {
            error: ErrorDetail {
                code: code.to_string(),
                message,
                details: None,
            },
        });

        (status, body).into_response()
    }
}

/// Mapea errores de dominio a códigos HTTP.
fn map_domain_error(e: &DomainError) -> (StatusCode, &'static str, String) {
    use DomainError::*;

    match e {
        NotFound { resource } => (
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            format!("{} not found", resource),
        ),
        EmailAlreadyExists { email } => (
            StatusCode::CONFLICT,
            "EMAIL_EXISTS",
            format!("Email '{}' already exists", email),
        ),
        InvalidEmail { reason } => (
            StatusCode::BAD_REQUEST,
            "INVALID_EMAIL",
            format!("Invalid email: {}", reason),
        ),
        InvalidPassword { reason } => (
            StatusCode::BAD_REQUEST,
            "INVALID_PASSWORD",
            format!("Invalid password: {}", reason),
        ),
        InvalidToken => (StatusCode::UNAUTHORIZED, "INVALID_TOKEN", "Invalid token".to_string()),
        InvalidCredentials => (
            StatusCode::UNAUTHORIZED,
            "INVALID_CREDENTIALS",
            "Invalid credentials".to_string(),
        ),
        Forbidden { message } => (
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            message.clone(),
        ),
        MissingPermission { permission } => (
            StatusCode::FORBIDDEN,
            "MISSING_PERMISSION",
            format!("Permission '{}' required", permission),
        ),
        InvalidPermission { reason } => (
            StatusCode::BAD_REQUEST,
            "INVALID_PERMISSION",
            format!("Invalid permission: {}", reason),
        ),
        InvalidId { message } => (
            StatusCode::BAD_REQUEST,
            "INVALID_ID",
            message.clone(),
        ),
        Database(msg) => {
            error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "Database error".to_string(),
            )
        }
        Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
        Internal(msg) => {
            error!("Internal domain error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error".to_string(),
            )
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        ApiError::Domain(e)
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        error!("SQLx error: {}", e);
        ApiError::Internal(e.to_string())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        error!("Anyhow error: {}", e);
        ApiError::Internal(e.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        error!("IO error: {}", e);
        ApiError::Internal(e.to_string())
    }
}

/// Tipo resultado de la API.
pub type ApiResult<T> = Result<T, ApiError>;
