// Ubicación: `crates/domain/src/entities/audit_log.rs`
//
// Descripción: Entidad AuditLog para auditoría de acciones.
//              Insert-only, nunca se modifica ni elimina.
//
// ADRs relacionados: ADR 0001, ADR 0006

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Entidad AuditLog del dominio.
/// Insert-only: nunca se modifica después de crear.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,         // UUID v7
    pub user_id: Option<String>, // Nullable (anon actions)
    pub action: String,     // formato: "resource:action" (ej: "users:create")
    pub resource: String,
    pub resource_id: Option<String>,
    pub details: Option<String>, // JSON con old/new values
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: OffsetDateTime,
}

impl AuditLog {
    /// Crea un nuevo registro de auditoría.
    pub fn new(
        user_id: Option<String>,
        action: String,
        resource: String,
        resource_id: Option<String>,
        details: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7().to_string(),
            user_id,
            action,
            resource,
            resource_id,
            details,
            ip_address,
            user_agent,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    /// Helper para crear log de usuario.
    pub fn for_user(
        user_id: String,
        action: &str,
        resource: &str,
        resource_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self::new(
            Some(user_id),
            action.to_string(),
            resource.to_string(),
            resource_id,
            None,
            ip_address,
            user_agent,
        )
    }
}
