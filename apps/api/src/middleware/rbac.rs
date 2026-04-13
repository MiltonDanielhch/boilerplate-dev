// Ubicación: `apps/api/src/middleware/rbac.rs`
//
// Descripción: Middleware de RBAC — verifica permisos del usuario autenticado.
//
// ADRs relacionados: ADR 0006 (RBAC), ADR 0017 (Caché)

use crate::error::ApiError;
use crate::middleware::auth::AuthClaims;
use crate::state::AppState;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use domain::{ports::UserRepository, value_objects::UserId};

/// Middleware RBAC que verifica un permiso específico.
///
/// Uso:
/// ```rust
/// Router::new()
///     .route("/api/v1/users", get(users::list))
///     .layer(middleware::from_fn_with_state(
///         state.clone(),
///         |state, req, next| rbac_middleware(state, req, next, "users:read")
///     ))
/// ```
///
/// Retorna 403 Forbidden si el usuario no tiene el permiso.
pub async fn rbac_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
    permission: &'static str,
) -> Result<Response, ApiError> {
    // Extraer user_id de las extensiones (inyectado por auth_middleware)
    let claims = request
        .extensions()
        .get::<AuthClaims>()
        .ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

    let user_id = UserId::parse(&claims.user_id)
        .map_err(|_| ApiError::Internal("Invalid user_id in token".to_string()))?;

    // Verificar permiso usando el repositorio
    let has_perm = state
        .user_repo
        .has_permission(&user_id, permission)
        .await
        .map_err(|e| ApiError::Internal(format!("Permission check failed: {}", e)))?;

    if !has_perm {
        return Err(ApiError::Forbidden(format!(
            "Permission denied: requires '{}'",
            permission
        )));
    }

    // Usuario tiene permiso, continuar
    Ok(next.run(request).await)
}

/// Middleware RBAC con permiso "users:read".
pub async fn require_users_read(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    rbac_middleware(State(state), request, next, "users:read").await
}

/// Middleware RBAC con permiso "users:write".
pub async fn require_users_write(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    rbac_middleware(State(state), request, next, "users:write").await
}

/// Middleware RBAC con permiso "audit:read" (para logs de auditoría).
pub async fn require_audit_read(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    rbac_middleware(State(state), request, next, "audit:read").await
}

/// Middleware RBAC con permiso "roles:read".
pub async fn require_roles_read(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    rbac_middleware(State(state), request, next, "roles:read").await
}

/// Middleware RBAC con permiso "roles:write".
pub async fn require_roles_write(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    rbac_middleware(State(state), request, next, "roles:write").await
}
