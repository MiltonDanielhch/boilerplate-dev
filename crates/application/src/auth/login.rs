// Ubicación: `crates/application/src/auth/login.rs`
//
// Descripción: Caso de uso Login.
//              Flujo: verify → PASETO 15min → refresh → session → audit.
//
// ADRs relacionados: ADR 0001, ADR 0008 (PASETO)

use domain::entities::User;
use domain::errors::DomainError;
use domain::ports::{AuditRepository, SessionRepository, UserRepository};

/// Input para login.
#[derive(Debug, Clone)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Output del login (tokens).
#[derive(Debug, Clone)]
pub struct LoginOutput {
    pub user: User,
    pub access_token: String,  // PASETO v4.local (15 min)
    pub refresh_token: String, // PASETO v4.local (7 días)
}

/// Caso de uso: Login de usuario.
pub struct LoginUseCase<R, S, A>
where
    R: UserRepository,
    S: SessionRepository,
    A: AuditRepository,
{
    user_repo: R,
    session_repo: S,
    audit_repo: A,
}

impl<R, S, A> LoginUseCase<R, S, A>
where
    R: UserRepository,
    S: SessionRepository,
    A: AuditRepository,
{
    pub fn new(user_repo: R, session_repo: S, audit_repo: A) -> Self {
        Self {
            user_repo,
            session_repo,
            audit_repo,
        }
    }

    /// Ejecuta el login.
    /// TODO: Verificar password con argon2.
    /// TODO: Generar PASETO tokens (v4.local).
    /// TODO: Crear sesión en DB.
    /// TODO: Registrar en audit_log.
    pub async fn execute(&self, _input: LoginInput) -> Result<LoginOutput, DomainError> {
        // Placeholder — implementación completa en fase de seguridad (Bloque III)
        Err(DomainError::Internal("Login no implementado — esperando Bloque III".to_string()))
    }
}
