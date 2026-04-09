// Ubicación: `crates/infrastructure/src/router.rs`
//
// Descripción: Definición de rutas de la API.
//
// ADRs relacionados: ADR 0003

use axum::Router;
use axum::routing::get;

pub fn routes() -> Router {
    Router::new().route("/health", get(|| async { "OK" }))
}
