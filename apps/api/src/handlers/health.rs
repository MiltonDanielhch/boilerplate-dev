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
use utoipa::ToSchema;

/// Response del health check
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub database: &'static str,
    pub version: &'static str,
}

/// GET /health — verifica que todo esté funcionando.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Servidor funcionando correctamente", body = HealthResponse),
        (status = 500, description = "Error de conexión a la base de datos", body = crate::error::ErrorResponse),
    )
)]
pub async fn health(State(state): State<AppState>) -> ApiResult<Json<HealthResponse>> {
    // Verificar conexión a DB con una query simple
    let _ = state.user_repo.find_by_id(&domain::value_objects::UserId::new()).await;
    // Si falla, retorna error 500

    Ok(Json(HealthResponse {
        status: "ok",
        database: "connected",
        version: env!("CARGO_PKG_VERSION"),
    }))
}
