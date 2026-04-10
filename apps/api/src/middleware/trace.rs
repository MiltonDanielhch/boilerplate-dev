// Ubicación: `apps/api/src/middleware/trace.rs`
//
// Descripción: Middleware simple para logging de requests.
//
// ADRs relacionados: ADR 0003 (Axum)

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::info;

/// Middleware function para request tracing.
pub async fn trace_middleware(
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let start = Instant::now();
    
    info!(%method, %path, "Request started");
    
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    let status = response.status().as_u16();
    
    info!(%status, ?duration, %method, %path, "Request completed");
    
    response
}
