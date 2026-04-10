// Ubicación: `crates/domain/src/ports/audit_repository.rs`
//
// Descripción: Puerto (trait) para persistencia de logs de auditoría.
//              Insert-only, nunca se modifica ni elimina.
//
// ADRs relacionados: ADR 0001, ADR 0006

use crate::entities::AuditLog;
use crate::errors::DomainError;
use async_trait::async_trait;
use time::OffsetDateTime;

/// Puerto para operaciones de auditoría (insert-only).
#[async_trait]
pub trait AuditRepository: Send + Sync {
    /// Registra un evento de auditoría.
    async fn log(&self, log: &AuditLog) -> Result<(), DomainError>;

    /// Busca logs por recurso.
    async fn find_by_resource(
        &self,
        resource: &str,
        resource_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<AuditLog>, DomainError>;

    /// Busca logs por usuario.
    async fn find_by_user(&self, user_id: &str, limit: i64) -> Result<Vec<AuditLog>, DomainError>;

    /// Busca logs por rango de fechas.
    async fn find_by_date_range(
        &self,
        from: OffsetDateTime,
        to: OffsetDateTime,
        limit: i64,
    ) -> Result<Vec<AuditLog>, DomainError>;
}
