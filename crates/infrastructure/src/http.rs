// Ubicación: `crates/infrastructure/src/http.rs`
//
// Descripción: Adaptadores HTTP con Axum.
//
// ADRs relacionados: ADR 0003

use axum::Router;

pub fn create_router() -> Router {
    Router::new()
}
