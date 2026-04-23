// Ubicación: `crates/application/src/auth/register.rs`
//
// Descripción: Caso de uso Registrar Usuario.
//              Flujo: email → argon2id → save → encola EmailJob.
//
// ADRs relacionados: ADR 0001, ADR 0008 (PASETO), ADR 0019 (Email)

use domain::entities::User;
use domain::errors::DomainError;
use domain::ports::{Mailer, UserRepository};
use domain::value_objects::{Email, PasswordHash};

/// Input para registro de usuario.
#[derive(Debug, Clone)]
pub struct RegisterInput {
    pub email: String,
    pub password: String, // Contraseña en texto plano (se hashea aquí)
    pub name: Option<String>,
}

/// Caso de uso: Registrar nuevo usuario.
pub struct RegisterUseCase<R, M>
where
    R: UserRepository,
    M: Mailer,
{
    _user_repo: R,
    _mailer: M,
}

impl<R, M> RegisterUseCase<R, M>
where
    R: UserRepository,
    M: Mailer,
{
    pub fn new(user_repo: R, mailer: M) -> Self {
        Self { _user_repo: user_repo, _mailer: mailer }
    }

    /// Ejecuta el registro.
    /// TODO: Integrar argon2 para hashear password antes de crear User.
    /// TODO: Verificar email duplicado antes de crear.
    /// TODO: Encolar EmailJob para verificación.
    pub async fn execute(&self, input: RegisterInput) -> Result<User, DomainError> {
        // Validar email
        let email = Email::new(&input.email)?;

        // Verificar si email ya existe
        if let Some(_existing) = self._user_repo.find_active_by_email(&email).await? {
            return Err(DomainError::EmailAlreadyExists {
                email: input.email,
            });
        }

        // TODO: Hashear password con argon2id (en infraestructura)
        // Por ahora usamos un placeholder que valida formato
        let password_hash = PasswordHash::new(&input.password)?;

        // Crear usuario
        let user = User::new(email, password_hash, input.name)?;

        // Guardar en repositorio
        self._user_repo.save(&user).await?;

        // TODO: Encolar EmailVerificationJob (no bloquear HTTP)
        // self.job_queue.enqueue(EmailVerificationJob { user_id: user.id }).await?;

        Ok(user)
    }
}
