// Ubicación: `crates/domain/src/ports/lead_repository.rs`
//
// Descripción: Puerto (trait) para persistencia de leads (landing page).
//
// ADRs relacionados: ADR 0001, ADR 0029

use crate::entities::Lead;
use crate::errors::DomainError;
use async_trait::async_trait;

/// Puerto para operaciones de persistencia de leads.
#[async_trait]
pub trait LeadRepository: Send + Sync {
    /// Guarda un nuevo lead.
    async fn save(&self, lead: &Lead) -> Result<(), DomainError>;

    /// Busca lead por email.
    async fn find_by_email(&self, email: &str) -> Result<Option<Lead>, DomainError>;

    /// Busca lead por ID.
    async fn find_by_id(&self, id: &str) -> Result<Option<Lead>, DomainError>;

    /// Lista leads con paginación.
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Lead>, DomainError>;

    /// Marca lead como contactado.
    async fn mark_contacted(&self, id: &str, notes: Option<&str>) -> Result<(), DomainError>;
}
