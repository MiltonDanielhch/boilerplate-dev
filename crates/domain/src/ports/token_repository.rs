// Ubicación: `crates/domain/src/ports/token_repository.rs`
//
// Descripción: Puerto (trait) para tokens de verificación (email, password reset).
//              Tokens de un solo uso.
//
// ADRs relacionados: ADR 0001

use crate::errors::DomainError;
use async_trait::async_trait;
use time::OffsetDateTime;

/// Puerto para operaciones con tokens de verificación.
#[async_trait]
pub trait TokenRepository: Send + Sync {
    /// Crea un nuevo token.
    async fn create(
        &self,
        user_id: &str,
        token_hash: &str,
        purpose: &str, // "email_verification" | "password_reset"
        expires_at: OffsetDateTime,
    ) -> Result<(), DomainError>;

    /// Usa un token (lo marca como usado). Retorna el user_id si válido.
    async fn use_token(&self, token_hash: &str, purpose: &str) -> Result<Option<String>, DomainError>;

    /// Verifica si un usuario tiene token activo de cierto propósito.
    async fn has_active_token(&self, user_id: &str, purpose: &str) -> Result<bool, DomainError>;

    /// Invalida todos los tokens de un usuario para un propósito.
    async fn invalidate_for_user(&self, user_id: &str, purpose: &str) -> Result<(), DomainError>;

    /// Limpia tokens expirados o usados.
    async fn cleanup(&self, before: OffsetDateTime) -> Result<u64, DomainError>;
}
