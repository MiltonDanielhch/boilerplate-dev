// Ubicación: `crates/domain/src/ports/session_repository.rs`
//
// Descripción: Puerto (trait) para persistencia de sesiones.
//
// ADRs relacionados: ADR 0001, ADR 0008 (PASETO)

use crate::entities::Session;
use crate::errors::DomainError;
use async_trait::async_trait;
use time::OffsetDateTime;

/// Puerto para operaciones de persistencia de sesiones.
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Crea una nueva sesión.
    async fn create(&self, session: &Session) -> Result<(), DomainError>;

    /// Busca sesión por refresh token hash.
    async fn find_by_token(&self, token_hash: &str) -> Result<Option<Session>, DomainError>;

    /// Busca todas las sesiones activas de un usuario.
    async fn find_active_by_user(&self, user_id: &str) -> Result<Vec<Session>, DomainError>;

    /// Revoca (invalida) una sesión.
    async fn revoke(&self, session_id: &str) -> Result<(), DomainError>;

    /// Revoca todas las sesiones de un usuario (logout global).
    async fn revoke_all_for_user(&self, user_id: &str) -> Result<(), DomainError>;

    /// Actualiza timestamp de última actividad.
    async fn update_activity(&self, session_id: &str) -> Result<(), DomainError>;

    /// Limpia sesiones expiradas (job de mantenimiento).
    async fn cleanup_expired(&self, before: OffsetDateTime) -> Result<u64, DomainError>;
}
