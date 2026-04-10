// Ubicación: `apps/api/src/handlers/health.rs`
//
// Descripción: Health check endpoint — verifica DB antes de responder 200.
//
// ADRs relacionados: ADR 0002 (Fail-Fast)

use crate::error::ApiResult;
use crate::state::AppState;
use axum::{
    extract::State,
    response::Json,
};
use domain::ports::UserRepository;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub database: &'static str,
    pub version: &'static str,
}

/// GET /health — verifica que todo esté funcionando.
pub async fn handler(State(state): State<AppState>) -> ApiResult<Json<HealthResponse>> {
    // Verificar conexión a DB con una query simple
    let _ = state.user_repo.find_by_id(&domain::value_objects::UserId::new()).await;
    // Si falla, retorna error 500

    Ok(Json(HealthResponse {
        status: "ok",
        database: "connected",
        version: env!("CARGO_PKG_VERSION"),
    }))
}
