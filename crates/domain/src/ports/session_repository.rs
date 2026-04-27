// Ubicación: `crates/domain/src/ports/session_repository.rs`
//
// Descripción: Puerto (trait) para persistencia de sesiones.
//
// ADRs relacionados: ADR 0001, ADR 0008 (PASETO)

use crate::entities::Session;
use crate::errors::DomainError;
use std::future::Future;
use time::OffsetDateTime;

/// Puerto para operaciones de persistencia de sesiones.
pub trait SessionRepository: Send + Sync {
    /// Crea una nueva sesión.
    fn create(&self, session: &Session) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Busca sesión por refresh token hash.
    fn find_by_token(&self, token_hash: &str) -> impl Future<Output = Result<Option<Session>, DomainError>> + Send;

    /// Busca todas las sesiones activas de un usuario.
    fn find_active_by_user(&self, user_id: &str) -> impl Future<Output = Result<Vec<Session>, DomainError>> + Send;

    /// Revoca (invalida) una sesión.
    fn revoke(&self, session_id: &str) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Revoca todas las sesiones de un usuario (logout global).
    fn revoke_all_for_user(&self, user_id: &str) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Actualiza timestamp de última actividad.
    fn update_activity(&self, session_id: &str) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Limpia sesiones expiradas (job de mantenimiento).
    fn cleanup_expired(&self, before: OffsetDateTime) -> impl Future<Output = Result<u64, DomainError>> + Send;

    /// Lista todas las sesiones activas del sistema (admin).
    fn list_all(&self, limit: i64, offset: i64) -> impl Future<Output = Result<Vec<Session>, DomainError>> + Send;
}
