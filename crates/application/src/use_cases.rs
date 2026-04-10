// Ubicación: `crates/application/src/use_cases.rs`
//
// Descripción: Casos de uso del sistema.
//
// ADRs relacionados: ADR 0001

use domain::entities::User;
use domain::errors::DomainError;
use domain::value_objects::{Email, PasswordHash};

pub struct CreateUserUseCase;

impl CreateUserUseCase {
    /// Ejecuta el caso de uso creando un nuevo usuario.
    /// En fase posterior usará repositorios para persistir.
    pub fn execute(
        &self,
        email: &str,
        password_hash: &str,
        name: Option<String>,
    ) -> Result<User, DomainError> {
        let email = Email::new(email)?;
        let password_hash = PasswordHash::new(password_hash)?;

        User::new(email, password_hash, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_hash() -> &'static str {
        "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$hash"
    }

    #[test]
    fn crea_usuario_con_email_valido() {
        let uc = CreateUserUseCase;
        let user = uc.execute("test@example.com", valid_hash(), None);
        assert!(user.is_ok());
    }

    #[test]
    fn error_con_email_invalido() {
        let uc = CreateUserUseCase;
        let result = uc.execute("invalid-email", valid_hash(), None);
        assert!(matches!(result, Err(DomainError::InvalidEmail { .. })));
    }

    #[test]
    fn error_con_password_hash_invalido() {
        let uc = CreateUserUseCase;
        let result = uc.execute("test@example.com", "not_argon2", None);
        assert!(matches!(result, Err(DomainError::InvalidPassword { .. })));
    }
}
