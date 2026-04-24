// Ubicación: `crates/application/src/auth/login.rs`
//
// Descripción: Caso de uso Login.
//              Flujo: verify → PASETO 15min → refresh → session → audit.
//
// ADRs relacionados: ADR 0001, ADR 0008 (PASETO)

use domain::entities::{Session, User};
use domain::errors::DomainError;
use domain::ports::{AuditRepository, PasswordVerifier, SessionRepository, TokenGenerator, UserRepository};
use domain::value_objects::Email;
use time::{Duration, OffsetDateTime};

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
pub struct LoginUseCase<'a, R, S, A, P, T>
where
    R: UserRepository,
    S: SessionRepository,
    A: AuditRepository,
    P: PasswordVerifier,
    T: TokenGenerator,
{
    user_repo: &'a R,
    session_repo: &'a S,
    _audit_repo: &'a A,
    password_verifier: &'a P,
    token_generator: &'a T,
}

impl<'a, R, S, A, P, T> LoginUseCase<'a, R, S, A, P, T>
where
    R: UserRepository,
    S: SessionRepository,
    A: AuditRepository,
    P: PasswordVerifier,
    T: TokenGenerator,
{
    pub fn new(
        user_repo: &'a R,
        session_repo: &'a S,
        audit_repo: &'a A,
        password_verifier: &'a P,
        token_generator: &'a T,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
            _audit_repo: audit_repo,
            password_verifier,
            token_generator,
        }
    }

    /// Ejecuta el login.
    pub async fn execute(&self, input: LoginInput) -> Result<LoginOutput, DomainError> {
        let email = Email::new(&input.email)
            .map_err(|e| DomainError::Validation(format!("Invalid email: {}", e)))?;

        let user = self.user_repo.find_active_by_email(&email).await?
            .ok_or_else(|| DomainError::InvalidCredentials)?;

        let valid = self.password_verifier.verify_password(&input.password, user.password_hash.as_str())?;
        if !valid {
            return Err(DomainError::InvalidCredentials);
        }

        let access_token = self.token_generator.generate_access_token(&user.id.uuid())?;
        let refresh_token = self.token_generator.generate_refresh_token();
        let refresh_token_hash = self.token_generator.hash_refresh_token(&refresh_token);

        let session = Session::new(
            user.id.to_string(),
            refresh_token_hash,
            input.ip_address,
            input.user_agent,
            OffsetDateTime::now_utc() + Duration::days(7),
        );

        self.session_repo.create(&session).await?;

        // TODO: Audit logging (requires action context)

        Ok(LoginOutput {
            user,
            access_token,
            refresh_token,
        })
    }
}
