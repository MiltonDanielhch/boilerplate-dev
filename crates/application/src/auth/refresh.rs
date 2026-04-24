// Ubicación: `crates/application/src/auth/refresh.rs`
//
// Descripción: Caso de uso Refresh Token.
//              Verifica refresh_token (hash) → revoca sesión → genera nuevo PASETO
//              + nuevo refresh token + crea nueva sesión (rotación obligatoria).
//
// ADRs relacionados: ADR 0001, ADR 0008 (PASETO)

use domain::errors::DomainError;
use domain::ports::{SessionRepository, TokenGenerator, UserRepository};
use domain::value_objects::UserId;
use time::{Duration, OffsetDateTime};
use domain::entities::Session;

/// Output del refresh.
#[derive(Debug, Clone)]
pub struct RefreshOutput {
    pub access_token: String,  // Nuevo PASETO (15 min)
    pub refresh_token: String, // Nuevo refresh opaco (7 días)
}

/// Caso de uso: Refresh de tokens con rotación obligatoria.
pub struct RefreshUseCase<'a, S, U, T>
where
    S: SessionRepository,
    U: UserRepository,
    T: TokenGenerator,
{
    session_repo: &'a S,
    user_repo: &'a U,
    token_generator: &'a T,
}

impl<'a, S, U, T> RefreshUseCase<'a, S, U, T>
where
    S: SessionRepository,
    U: UserRepository,
    T: TokenGenerator,
{
    pub fn new(session_repo: &'a S, user_repo: &'a U, token_generator: &'a T) -> Self {
        Self { session_repo, user_repo, token_generator }
    }

    /// Ejecuta la rotación de tokens.
    /// Ref: ADR 0008 — rotación obligatoria en cada refresh.
    pub async fn execute(&self, refresh_token_raw: &str) -> Result<RefreshOutput, DomainError> {
        // Hashear el token recibido para buscarlo en DB
        let refresh_hash = self.token_generator.hash_refresh_token(refresh_token_raw);

        // Buscar sesión activa
        let session = self.session_repo.find_by_token(&refresh_hash).await?
            .ok_or(DomainError::InvalidToken)?;

        if session.is_revoked {
            return Err(DomainError::InvalidToken);
        }
        if session.is_expired() {
            return Err(DomainError::InvalidToken);
        }

        // Obtener usuario asociado
        let user_id = UserId::parse(&session.user_id)
            .map_err(|e| DomainError::InvalidId { message: e.to_string() })?;
        let user = self.user_repo.find_by_id(&user_id).await?
            .ok_or(DomainError::InvalidCredentials)?;

        // Revocar sesión antigua (rotación obligatoria)
        self.session_repo.revoke(&session.id).await?;

        // Generar nuevos tokens
        let new_access_token = self.token_generator.generate_access_token(&user.id.uuid())?;
        let new_refresh_raw = self.token_generator.generate_refresh_token();
        let new_refresh_hash = self.token_generator.hash_refresh_token(&new_refresh_raw);

        // Crear nueva sesión
        let new_session = Session::new(
            session.user_id,
            new_refresh_hash,
            session.ip_address,
            session.user_agent,
            OffsetDateTime::now_utc() + Duration::days(7),
        );
        self.session_repo.create(&new_session).await?;

        Ok(RefreshOutput {
            access_token: new_access_token,
            refresh_token: new_refresh_raw,
        })
    }
}
