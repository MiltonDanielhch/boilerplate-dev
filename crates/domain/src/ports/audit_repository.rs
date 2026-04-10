// Ubicación: `crates/domain/src/ports/audit_repository.rs`
//
// Descripción: Puerto (trait) para persistencia de logs de auditoría.
//              Insert-only, nunca se modifica ni elimina.
//
// ADRs relacionados: ADR 0001, ADR 0006

use crate::entities::AuditLog;
use crate::errors::DomainError;
use std::future::Future;
use time::OffsetDateTime;

/// Puerto para operaciones de auditoría (insert-only).
pub trait AuditRepository: Send + Sync {
    /// Registra un evento de auditoría.
    fn log(&self, log: &AuditLog) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Busca logs por recurso.
    fn find_by_resource(
        &self,
        resource: &str,
        resource_id: Option<&str>,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<AuditLog>, DomainError>> + Send;

    /// Busca logs por usuario.
    fn find_by_user(&self, user_id: &str, limit: i64) -> impl Future<Output = Result<Vec<AuditLog>, DomainError>> + Send;

    /// Busca logs por rango de fechas.
    fn find_by_date_range(
        &self,
        from: OffsetDateTime,
        to: OffsetDateTime,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<AuditLog>, DomainError>> + Send;
}
