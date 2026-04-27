// Ubicación: `apps/api/src/handlers/audit.rs`
//
// Descripción: Handlers para endpoints de auditoría.
//
// ADRs relacionados: ADR 0003 (Axum), ADR 0006 (Audit)

use crate::error::ApiResult;
use crate::state::AppState;
use axum::{
    extract::State,
    response::Json,
};
use domain::ports::AuditRepository;
use application::audit::{ListAuditUseCase, ListAuditInput};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub user_id: Option<String>,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl From<domain::entities::AuditLog> for AuditEntry {
    fn from(log: domain::entities::AuditLog) -> Self {
        Self {
            id: log.id,
            timestamp: log.created_at.to_string(),
            user_id: log.user_id,
            action: log.action,
            resource: log.resource,
            resource_id: log.resource_id,
            details: log.details,
            ip_address: log.ip_address,
            user_agent: log.user_agent,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAuditQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub user_id: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub from_date: Option<time::OffsetDateTime>,
    pub to_date: Option<time::OffsetDateTime>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListAuditResponse {
    pub entries: Vec<AuditEntry>,
    pub total: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/audit",
    tag = "Audit",
    responses(
        (status = 200, description = "Lista de entradas de auditoría", body = ListAuditResponse),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "Sin permiso audit:read"),
    )
)]
pub async fn list_audit(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ListAuditQuery>,
) -> ApiResult<Json<ListAuditResponse>> {
    let use_case = ListAuditUseCase::new(state.audit_repo);
    let input = ListAuditInput {
        limit: query.limit.unwrap_or(50).min(100),
        offset: query.offset.unwrap_or(0),
        user_id: query.user_id,
        resource: query.resource,
        action: query.action,
        from_date: query.from_date,
        to_date: query.to_date,
    };
    
    let entries = use_case.execute(input).await?;
    let total = entries.len() as i64; // TODO: Real count
    let entries: Vec<AuditEntry> = entries.into_iter().map(AuditEntry::from).collect();

    Ok(Json(ListAuditResponse { entries, total }))
}

#[utoipa::path(
    get,
    path = "/api/v1/audit/recent",
    tag = "Audit",
    responses(
        (status = 200, description = "Entradas recientes", body = Vec<AuditEntry>),
        (status = 401, description = "No autenticado"),
    )
)]
pub async fn recent_audit(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<AuditEntry>>> {
    let entries = state.audit_repo
        .find_by_resource("api", None, 20)
        .await
        .map_err(|e| crate::error::ApiError::Internal(e.to_string()))?;

    let entries: Vec<AuditEntry> = entries.into_iter().map(AuditEntry::from).collect();

    Ok(Json(entries))
}