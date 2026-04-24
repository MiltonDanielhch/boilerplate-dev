// Ubicación: `crates/application/src/auth/register.rs`
//
// Descripción: Caso de uso Registrar Usuario.
//              Flujo: email → argon2id → save → audit → encola EmailJob.
//
// ADRs relacionados: ADR 0001, ADR 0008 (PASETO), ADR 0019 (Email)

use domain::entities::{User, AuditLog};
use domain::errors::DomainError;
use domain::ports::{AuditRepository, Mailer, PasswordHasher, UserRepository};
use domain::value_objects::{Email, PasswordHash};

/// Input para registro de usuario.
#[derive(Debug, Clone)]
pub struct RegisterInput {
    pub email: String,
    pub password: String, // Contraseña en texto plano
    pub name: Option<String>,
}

/// Caso de uso: Registrar nuevo usuario.
pub struct RegisterUseCase<'a, R, M, H, A>
where
    R: UserRepository,
    M: Mailer,
    H: PasswordHasher,
    A: AuditRepository,
{
    user_repo: &'a R,
    _mailer: &'a M,
    password_hasher: &'a H,
    audit_repo: &'a A,
}

impl<'a, R, M, H, A> RegisterUseCase<'a, R, M, H, A>
where
    R: UserRepository,
    M: Mailer,
    H: PasswordHasher,
    A: AuditRepository,
{
    pub fn new(user_repo: &'a R, mailer: &'a M, password_hasher: &'a H, audit_repo: &'a A) -> Self {
        Self {
            user_repo,
            _mailer: mailer,
            password_hasher,
            audit_repo,
        }
    }

    /// Ejecuta el registro.
    pub async fn execute(&self, input: RegisterInput) -> Result<User, DomainError> {
        // Validar email
        let email = Email::new(&input.email)?;

        // Verificar si email ya existe
        if let Some(_existing) = self.user_repo.find_active_by_email(&email).await? {
            return Err(DomainError::EmailAlreadyExists {
                email: input.email,
            });
        }

        // Hashear password con argon2id
        let password_hash_str = self.password_hasher.hash_password(&input.password)?;
        let password_hash = PasswordHash::new(&password_hash_str)?;

        // Crear usuario
        let user = User::new(email, password_hash, input.name)?;

        // Guardar en repositorio
        self.user_repo.save(&user).await?;

        // Registrar en auditoría
        let audit = AuditLog::new(
            Some(user.id.to_string()),
            "user.register".to_string(),
            "User".to_string(),
            Some(user.id.to_string()),
            Some(serde_json::to_string(&serde_json::json!({ "email": user.email.value() })).unwrap_or_default()),
            None, // ip
            None, // ua
        );
        self.audit_repo.log(&audit).await?;

        // TODO: Encolar EmailVerificationJob
        // self._mailer.send_verification_email(...).await?;

        Ok(user)
    }
}

