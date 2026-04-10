// Ubicación: `crates/application/src/auth/refresh.rs`
//
// Descripción: Caso de uso Refresh Token.
//              Verifica refresh → nuevo PASETO + nuevo refresh.
//
// ADRs relacionados: ADR 0001, ADR 0008 (PASETO)

use domain::errors::DomainError;
use domain::ports::SessionRepository;

/// Output del refresh.
#[derive(Debug, Clone)]
pub struct RefreshOutput {
    pub access_token: String,  // Nuevo PASETO (15 min)
    pub refresh_token: String, // Nuevo refresh token (7 días)
}

/// Caso de uso: Refresh de tokens.
pub struct RefreshUseCase<S: SessionRepository> {
    session_repo: S,
}

impl<S: SessionRepository> RefreshUseCase<S> {
    pub fn new(session_repo: S) -> Self {
        Self { session_repo }
    }

    /// Ejecuta el refresh.
    /// TODO: Verificar refresh token PASETO.
    /// TODO: Rotar tokens (nuevo access + nuevo refresh).
    /// TODO: Invalidar token anterior.
    pub async fn execute(&self, _refresh_token: &str) -> Result<RefreshOutput, DomainError> {
        // Placeholder — implementación completa en fase de seguridad (Bloque III)
        Err(DomainError::Internal(
            "Refresh no implementado — esperando Bloque III".to_string(),
        ))
    }
}
