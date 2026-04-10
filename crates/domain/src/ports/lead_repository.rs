// Ubicación: `crates/domain/src/ports/lead_repository.rs`
//
// Descripción: Puerto (trait) para persistencia de leads (landing page).
//
// ADRs relacionados: ADR 0001, ADR 0029

use crate::entities::Lead;
use crate::errors::DomainError;
use std::future::Future;

/// Puerto para operaciones de persistencia de leads.
pub trait LeadRepository: Send + Sync {
    /// Guarda un nuevo lead.
    fn save(&self, lead: &Lead) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Busca lead por email.
    fn find_by_email(&self, email: &str) -> impl Future<Output = Result<Option<Lead>, DomainError>> + Send;

    /// Busca lead por ID.
    fn find_by_id(&self, id: &str) -> impl Future<Output = Result<Option<Lead>, DomainError>> + Send;

    /// Lista leads con paginación.
    fn list(&self, limit: i64, offset: i64) -> impl Future<Output = Result<Vec<Lead>, DomainError>> + Send;

    /// Marca lead como contactado.
    fn mark_contacted(&self, id: &str, notes: Option<&str>) -> impl Future<Output = Result<(), DomainError>> + Send;
}
