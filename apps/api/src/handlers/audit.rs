use crate::error::ApiResult;
use crate::state::AppState;
use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditEntry {
    pub timestamp: String,
    #[schema(value_type = String)]
    pub method: String,
    pub uri: String,
    pub status: i32,
    pub user_id: Option<String>,
    pub ip: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListAuditQuery {
    pub limit: Option<i64>,
    pub user_id: Option<String>,
    pub resource: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListAuditResponse {
    pub entries: Vec<AuditEntry>,
    pub total: i64,
}

fn create_audit_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/audit", get(list_audit))
        .route("/api/v1/audit/recent", get(recent_audit))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/v1/audit",
    tag = "Audit",
    responses(
        (status = 200, description = "Lista de entradas de auditoría"),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "Sin permiso audit:read"),
    )
)]
pub async fn list_audit(
    State(_state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ListAuditQuery>,
) -> ApiResult<Json<ListAuditResponse>> {
    let _limit = query.limit.unwrap_or(50).min(100);
    
    Ok(Json(ListAuditResponse {
        entries: vec![],
        total: 0,
    }))
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
    State(_state): State<AppState>,
) -> ApiResult<Json<Vec<AuditEntry>>> {
    Ok(Json(vec![]))
}