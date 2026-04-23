// Ubicación: `apps/api/src/router.rs`
//
// Descripción: Router modular con middleware.
//
// ADRs relacionados: ADR 0003 (Axum), ADR 0009 (Rate Limit), ADR 0021 (OpenAPI)

use crate::docs::ApiDoc;
use crate::handlers::{audit, auth, health, leads, users};
use crate::middleware::audit::audit_middleware;
use crate::middleware::auth::auth_middleware;
use crate::middleware::rbac::{require_audit_read, require_users_read, require_users_write};
use crate::middleware::request_id::request_id_middleware;
use crate::middleware::trace::trace_middleware;
use crate::state::AppState;
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use std::time::Duration;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

/// Crea el router con todos los endpoints y middleware.
pub fn create_router(state: AppState) -> Router {
    // Router público (sin autenticación)
    let public_routes = Router::new()
        .route("/health", get(health::health))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/logout", post(auth::logout))
        .route("/api/v1/leads", post(leads::capture))
        // Documentación API - disponible en todos los entornos
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
        .route("/openapi.json", get(|| async { axum::Json(ApiDoc::openapi()) }));

    // Router de usuarios con auth + RBAC
    // users:read para GET, users:write para POST/PUT/DELETE
    // Orden de middleware: auth primero (extrae claims), luego RBAC (verifica permisos)
    let users_read_routes = Router::new()
        .route("/api/v1/users", get(users::list))
        .route("/api/v1/users/{id}", get(users::get))
        .layer(middleware::from_fn_with_state(state.clone(), require_users_read))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let users_write_routes = Router::new()
        .route("/api/v1/users", post(users::create))
        .route("/api/v1/users/{id}", put(users::update).delete(users::soft_delete))
        .layer(middleware::from_fn_with_state(state.clone(), require_users_write))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Router protegido general (solo auth, sin RBAC específico)
    let protected_routes = Router::new();

    // Router de auditoría (auth + permission)
    let audit_routes = Router::new()
        .route("/api/v1/audit", get(audit::list_audit))
        .route("/api/v1/audit/recent", get(audit::recent_audit))
        .layer(middleware::from_fn_with_state(state.clone(), require_audit_read))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Combinar routers
    Router::new()
        .merge(public_routes)
        .merge(users_read_routes)
        .merge(users_write_routes)
        .merge(protected_routes)
        .merge(audit_routes)
        // State compartido
        .with_state(state.clone())
        // Middleware tower (orden: outer → inner)
        .layer(TimeoutLayer::new(Duration::from_secs(30))) // Timeout 30s
        .layer(CompressionLayer::new()) // Gzip/Brotli
        .layer(CorsLayer::permissive()) // CORS (restringir en prod)
        // Middleware axum functions
        .layer(middleware::from_fn_with_state(state, audit_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(trace_middleware))
}
