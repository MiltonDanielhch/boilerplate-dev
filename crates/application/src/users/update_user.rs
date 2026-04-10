// Ubicación: `crates/application/src/users/update_user.rs`
//
// Descripción: Caso de uso Actualizar Usuario.
//
// ADRs relacionados: ADR 0001

use domain::entities::User;
use domain::errors::DomainError;
use domain::ports::UserRepository;
use domain::value_objects::UserId;

/// Input para actualización.
#[derive(Debug, Clone, Default)]
pub struct UpdateUserInput {
    pub name: Option<String>,
    // TODO: Agregar más campos actualizables según requerimientos
}

/// Caso de uso: Actualizar usuario.
pub struct UpdateUserUseCase<R: UserRepository> {
    user_repo: R,
}

impl<R: UserRepository> UpdateUserUseCase<R> {
    pub fn new(user_repo: R) -> Self {
        Self { user_repo }
    }

    /// Actualiza datos del usuario.
    pub async fn execute(
        &self,
        id: &UserId,
        input: UpdateUserInput,
    ) -> Result<User, DomainError> {
        let mut user = self
            .user_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::not_found("User"))?;

        if !user.is_active() {
            return Err(DomainError::not_found("User"));
        }

        // Aplicar cambios
        if input.name.is_some() {
            user.set_name(input.name);
        }

        // Guardar cambios
        self.user_repo.save(&user).await?;

        Ok(user)
    }
}
