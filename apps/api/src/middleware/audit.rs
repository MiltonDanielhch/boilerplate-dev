// Ubicación: `apps/api/src/middleware/audit.rs`
//
// Descripción: Middleware de auditoría — logging fire-and-forget de requests HTTP.
//              No bloquea la respuesta (spawn + channel opcional).
//
// ADRs relacionados: ADR 0006 (RBAC), ADR 0002 (Fail-Fast)

use axum::{
    extract::ConnectInfo,
    http::{Method, Uri},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;
use std::time::Instant;
use tracing::{info, warn};

/// Información de auditoría de un request HTTP.
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub timestamp: String,
    pub method: String,
    pub uri: String,
    pub status_code: u16,
    pub duration_ms: u64,
    pub client_ip: String,
    pub user_agent: String,
    pub user_id: Option<String>,
}

impl AuditEvent {
    /// Serializa el evento para persistencia (JSON Lines format).
    pub fn to_json_line(&self) -> String {
        format!(
            r#"{{"timestamp":"{}","method":"{}","uri":"{}","status":{},"duration_ms":{},"ip":"{}","ua":"{}","user_id":{}}}"#,
            self.timestamp,
            self.method,
            self.uri,
            self.status_code,
            self.duration_ms,
            self.client_ip,
            self.user_agent.replace('"', "\\\""),
            self.user_id.as_deref().map(|s| format!("\"{}\"", s)).unwrap_or_else(|| "null".to_string())
        )
    }
}

/// Middleware de auditoría fire-and-forget.
///
/// Captura información del request y lo loguea de forma asíncrona.
/// No bloquea la respuesta HTTP.
///
/// # Uso
/// ```rust
/// router.layer(middleware::from_fn(audit_middleware))
/// ```
pub async fn audit_middleware(
    method: Method,
    uri: Uri,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    
    // Extraer user_id si existe en extensions (inyectado por auth_middleware)
    let user_id = request
        .extensions()
        .get::<crate::middleware::auth::AuthClaims>()
        .map(|claims| claims.user_id.clone());
    
    // Extraer User-Agent
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    // Ejecutar el request
    let response = next.run(request).await;
    
    // Calcular duración y capturar status
    let duration = start.elapsed();
    let status = response.status();
    
    // Crear evento de auditoría
    let event = AuditEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        method: method.to_string(),
        uri: uri.to_string(),
        status_code: status.as_u16(),
        duration_ms: duration.as_millis() as u64,
        client_ip: addr.ip().to_string(),
        user_agent,
        user_id,
    };
    
    // Fire-and-forget: Loguear asíncronamente sin bloquear
    // En producción, esto podría enviar a un channel/buffer para batch insert
    let status_code = event.status_code;
    let log_line = event.to_json_line();
    
    tokio::spawn(async move {
        if status_code >= 200 && status_code < 300 {
            info!(target: "audit", "{}", log_line);
        } else {
            warn!(target: "audit", "{}", log_line);
        }
    });
    
    response
}

/// Middleware de auditoría simplificado (solo para rutas críticas).
///
/// Similar a `audit_middleware` pero solo loguea requests exitosos (2xx).
pub async fn audit_success_only_middleware(
    method: Method,
    uri: Uri,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    
    let user_id = request
        .extensions()
        .get::<crate::middleware::auth::AuthClaims>()
        .map(|claims| claims.user_id.clone());
    
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    let response = next.run(request).await;
    let duration = start.elapsed();
    let status = response.status();
    
    // Solo auditar respuestas exitosas
    let status_code = status.as_u16();
    if status_code >= 200 && status_code < 300 {
        let event = AuditEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            method: method.to_string(),
            uri: uri.to_string(),
            status_code,
            duration_ms: duration.as_millis() as u64,
            client_ip: addr.ip().to_string(),
            user_agent,
            user_id,
        };
        
        let log_line = event.to_json_line();
        tokio::spawn(async move {
            info!(target: "audit", "{}", log_line);
        });
    }
    
    response
}
