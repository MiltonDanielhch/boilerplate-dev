// Ubicación: `apps/api/src/handlers/admin.rs`
// 
// Descripción: Handlers para el área de administración general.
// 
// ADRs relacionados: ADR 0001 (Hexagonal), ADR 0006 (RBAC)

use axum::{extract::State, Json};
use serde::Serialize;
use crate::error::ApiError;
use crate::middleware::auth::AuthClaims;
use crate::state::AppState;
use domain::{ports::UserRepository, value_objects::UserId};
use domain::ports::SessionRepository;
use application::admin::{GetAdminStatsUseCase, AdminStats, GetAdminAnalyticsUseCase, AdminAnalytics};

#[derive(Debug, Serialize)]
pub struct AdminMeResponse {
    pub is_admin: bool,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

/// Retorna información de administración del usuario actual.
/// Útil para que el frontend decida qué partes del Admin Dashboard mostrar.
pub async fn admin_me(
    State(state): State<AppState>,
    claims: AuthClaims,
) -> Result<Json<AdminMeResponse>, ApiError> {
    let user_id = UserId::parse(&claims.user_id)
        .map_err(|_| ApiError::Internal("Invalid user_id in token".to_string()))?;

    // En un sistema real, podríamos tener un método get_roles en el repo
    // Por ahora, verificamos admin y superadmin manualmente
    let mut roles = Vec::new();
    if state.user_repo.has_role(&user_id, "admin").await? {
        roles.push("admin".to_string());
    }
    if state.user_repo.has_role(&user_id, "superadmin").await? {
        roles.push("superadmin".to_string());
    }

    let permissions = state.user_repo.get_permissions(&user_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(AdminMeResponse {
        is_admin: !roles.is_empty(),
        roles,
        permissions,
    }))
}

/// GET /api/v1/admin/stats
pub async fn stats(
    State(state): State<AppState>,
    _claims: AuthClaims,
) -> Result<Json<AdminStats>, ApiError> {
    let use_case = GetAdminStatsUseCase::new(state.user_repo, state.lead_repo);
    let stats = use_case.execute().await?;
    Ok(Json(stats))
}

#[derive(serde::Deserialize)]
pub struct AnalyticsQuery {
    pub days: Option<i64>,
}

/// GET /api/v1/admin/analytics
pub async fn analytics(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<AnalyticsQuery>,
    _claims: AuthClaims,
) -> Result<Json<AdminAnalytics>, ApiError> {
    let days = query.days.unwrap_or(30);
    let use_case = GetAdminAnalyticsUseCase::new(state.user_repo, state.lead_repo);
    let data = use_case.execute(days).await?;
    Ok(Json(data))
}

/// GET /api/v1/admin/sessions
pub async fn list_sessions(
    State(state): State<AppState>,
    _claims: AuthClaims,
) -> Result<Json<Vec<SessionResponse>>, ApiError> {
    let sessions = state.session_repo.list_all(50, 0).await?;
    Ok(Json(sessions.into_iter().map(SessionResponse::from).collect()))
}

/// DELETE /api/v1/admin/sessions/{id}
pub async fn revoke_session(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    _claims: AuthClaims,
) -> Result<Json<()>, ApiError> {
    state.session_repo.revoke(&id).await?;
    Ok(Json(()))
}

#[derive(serde::Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub user_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_activity_at: String,
    pub expires_at: String,
}

impl From<domain::entities::Session> for SessionResponse {
    fn from(s: domain::entities::Session) -> Self {
        Self {
            id: s.id,
            user_id: s.user_id,
            ip_address: s.ip_address,
            user_agent: s.user_agent,
            last_activity_at: s.last_activity_at.to_string(),
            expires_at: s.expires_at.to_string(),
        }
    }
}
