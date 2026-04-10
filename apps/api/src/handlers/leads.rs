// Ubicación: `apps/api/src/handlers/leads.rs`
//
// Descripción: Handler para captura de leads desde landing page.
//
// ADRs relacionados: ADR 0003 (Axum), ADR 0029 (Landing + Leads)

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};

/// POST /api/v1/leads — Captura lead (rate limit 3/min en prod)
pub async fn capture(
    State(_state): State<AppState>,
    Json(_body): Json<CaptureLeadRequest>,
) -> ApiResult<Json<CaptureLeadResponse>> {
    // TODO: Implementar cuando tengamos SqliteLeadRepository (Bloque III)
    // Por ahora retornar 501 Not Implemented
    Err(ApiError::Internal(
        "Lead capture not yet implemented — pending LeadRepository".to_string()
    ))
}

#[derive(Debug, Deserialize)]
pub struct CaptureLeadRequest {
    pub email: String,
    pub name: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CaptureLeadResponse {
    pub message: String,
}
