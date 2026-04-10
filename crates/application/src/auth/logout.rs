// Ubicación: `crates/application/src/auth/logout.rs`
//
// Descripción: Caso de uso Logout.
//              Revoca sesión + refresh token.
//
// ADRs relacionados: ADR 0001, ADR 0008

use domain::errors::DomainError;
use domain::ports::SessionRepository;

/// Caso de uso: Logout (revocar sesión).
pub struct LogoutUseCase<S: SessionRepository> {
    session_repo: S,
}

impl<S: SessionRepository> LogoutUseCase<S> {
    pub fn new(session_repo: S) -> Self {
        Self { session_repo }
    }

    /// Revoca la sesión asociada al refresh token.
    pub async fn execute(&self, refresh_token_hash: &str) -> Result<(), DomainError> {
        if let Some(session) = self.session_repo.find_by_token(refresh_token_hash).await? {
            self.session_repo.revoke(&session.id).await?;
        }
        Ok(())
    }
}
