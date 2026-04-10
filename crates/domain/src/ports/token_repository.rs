// Ubicación: `crates/domain/src/ports/token_repository.rs`
//
// Descripción: Puerto (trait) para tokens de verificación (email, password reset).
//              Tokens de un solo uso.
//
// ADRs relacionados: ADR 0001

use crate::errors::DomainError;
use std::future::Future;
use time::OffsetDateTime;

/// Puerto para operaciones con tokens de verificación.
pub trait TokenRepository: Send + Sync {
    /// Crea un nuevo token.
    fn create(
        &self,
        user_id: &str,
        token_hash: &str,
        purpose: &str, // "email_verification" | "password_reset"
        expires_at: OffsetDateTime,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Usa un token (lo marca como usado). Retorna el user_id si válido.
    fn use_token(&self, token_hash: &str, purpose: &str) -> impl Future<Output = Result<Option<String>, DomainError>> + Send;

    /// Verifica si un usuario tiene token activo de cierto propósito.
    fn has_active_token(&self, user_id: &str, purpose: &str) -> impl Future<Output = Result<bool, DomainError>> + Send;

    /// Invalida todos los tokens de un usuario para un propósito.
    fn invalidate_for_user(&self, user_id: &str, purpose: &str) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Limpia tokens expirados o usados.
    fn cleanup(&self, before: OffsetDateTime) -> impl Future<Output = Result<u64, DomainError>> + Send;
}
