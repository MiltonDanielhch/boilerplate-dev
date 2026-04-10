// Ubicación: `crates/application/src/users/list_users.rs`
//
// Descripción: Caso de uso Listar Usuarios (paginado).
//
// ADRs relacionados: ADR 0001

use domain::entities::User;
use domain::errors::DomainError;
use domain::ports::UserRepository;

/// Input para listado (paginación).
#[derive(Debug, Clone)]
pub struct ListUsersInput {
    pub limit: i64,
    pub offset: i64,
}

impl Default for ListUsersInput {
    fn default() -> Self {
        Self {
            limit: 20,
            offset: 0,
        }
    }
}

/// Caso de uso: Listar usuarios.
pub struct ListUsersUseCase<R: UserRepository> {
    user_repo: R,
}

impl<R: UserRepository> ListUsersUseCase<R> {
    pub fn new(user_repo: R) -> Self {
        Self { user_repo }
    }

    /// Lista usuarios paginados.
    pub async fn execute(&self, input: ListUsersInput) -> Result<Vec<User>, DomainError> {
        self.user_repo.list(input.limit, input.offset).await
    }
}
