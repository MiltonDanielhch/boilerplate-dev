// Ubicación: `apps/api/src/router.rs`
//
// Descripción: Router modular con middleware.
//
// ADRs relacionados: ADR 0003 (Axum), ADR 0009 (Rate Limit)

use crate::handlers::{health, leads, users};
use crate::middleware::request_id::request_id_middleware;
use crate::middleware::trace::trace_middleware;
use crate::state::AppState;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    timeout::TimeoutLayer,
};
use std::time::Duration;

/// Crea el router con todos los endpoints y middleware.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health::handler))
        // Auth endpoints
        // .route("/auth/register", post(auth::register))
        // .route("/auth/login", post(auth::login))
        // .route("/auth/refresh", post(auth::refresh))
        // .route("/auth/logout", post(auth::logout))
        // User endpoints
        .route("/api/v1/users", get(users::list).post(users::create))
        .route(
            "/api/v1/users/:id",
            get(users::get).put(users::update).delete(users::soft_delete),
        )
        // Lead endpoints
        .route("/api/v1/leads", post(leads::capture))
        // State
        .with_state(state)
        // Middleware tower (orden: outer → inner)
        .layer(TimeoutLayer::new(Duration::from_secs(30))) // Timeout 30s
        .layer(CompressionLayer::new()) // Gzip/Brotli
        .layer(CorsLayer::permissive()) // CORS (restringir en prod)
        // Middleware axum functions
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(trace_middleware))
}
