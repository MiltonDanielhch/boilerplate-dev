// Ubicación: `crates/application/src/users/get_user.rs`
//
// Descripción: Caso de uso Obtener Usuario por ID.
//
// ADRs relacionados: ADR 0001

use domain::entities::User;
use domain::errors::DomainError;
use domain::ports::UserRepository;
use domain::value_objects::UserId;

/// Caso de uso: Obtener usuario por ID.
pub struct GetUserUseCase<R: UserRepository> {
    user_repo: R,
}

impl<R: UserRepository> GetUserUseCase<R> {
    pub fn new(user_repo: R) -> Self {
        Self { user_repo }
    }

    /// Busca usuario por ID. Retorna NotFound si no existe o está soft-deleted.
    pub async fn execute(&self, id: &UserId) -> Result<User, DomainError> {
        match self.user_repo.find_by_id(id).await? {
            Some(user) if user.is_active() => Ok(user),
            _ => Err(DomainError::not_found("User")),
        }
    }
}
