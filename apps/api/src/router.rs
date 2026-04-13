// Ubicación: `apps/api/src/router.rs`
//
// Descripción: Router modular con middleware.
//
// ADRs relacionados: ADR 0003 (Axum), ADR 0009 (Rate Limit)

use crate::handlers::{auth, health, leads, users};
use crate::middleware::auth::auth_middleware;
use crate::middleware::request_id::request_id_middleware;
use crate::middleware::trace::trace_middleware;
use crate::state::AppState;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::time::Duration;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer};

/// Crea el router con todos los endpoints y middleware.
pub fn create_router(state: AppState) -> Router {
    // Router público (sin autenticación)
    let public_routes = Router::new()
        .route("/health", get(health::handler))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout));

    // Router protegido (requiere Bearer token válido)
    let protected_routes = Router::new()
        .route("/api/v1/users", get(users::list).post(users::create))
        .route(
            "/api/v1/users/{id}",
            get(users::get)
                .put(users::update)
                .delete(users::soft_delete),
        )
        .route("/api/v1/leads", post(leads::capture))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Combinar routers
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // State compartido
        .with_state(state)
        // Middleware tower (orden: outer → inner)
        .layer(TimeoutLayer::new(Duration::from_secs(30))) // Timeout 30s
        .layer(CompressionLayer::new()) // Gzip/Brotli
        .layer(CorsLayer::permissive()) // CORS (restringir en prod)
        // Middleware axum functions
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(trace_middleware))
}
