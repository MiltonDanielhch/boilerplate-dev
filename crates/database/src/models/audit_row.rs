// Ubicación: `crates/database/src/models/audit_row.rs`
//
// Descripción: Modelo de fila para audit_logs.
//              Mapeo exacto de columnas de la tabla.
//
// ADRs relacionados: ADR 0001, ADR 0004

use domain::entities::AuditLog;
use domain::errors::DomainError;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, FromRow)]
pub struct AuditRow {
    pub id: String,
    pub user_id: Option<String>,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<String>,
    pub created_at: OffsetDateTime,
}

impl AuditRow {
    pub fn into_audit_log(self) -> Result<AuditLog, DomainError> {
        Ok(AuditLog {
            id: self.id,
            user_id: self.user_id,
            action: self.action,
            resource: self.resource,
            resource_id: self.resource_id,
            details: self.details,
            ip_address: self.ip_address,
            user_agent: self.user_agent,
            created_at: self.created_at,
        })
    }
}
