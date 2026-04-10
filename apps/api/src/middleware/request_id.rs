// Ubicación: `apps/api/src/middleware/request_id.rs`
//
// Descripción: Middleware para generar/propagar Request ID.
//
// ADRs relacionados: ADR 0003 (Axum)

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Middleware function para request ID.
pub async fn request_id_middleware(
    req: Request,
    next: Next,
) -> Response {
    // Generar o usar request ID existente
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Ejecutar siguiente middleware/handler
    let mut response = next.run(req).await;
    
    // Añadir request_id a la respuesta
    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );
    
    response
}
