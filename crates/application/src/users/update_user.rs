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
    pub is_active: Option<bool>,
    pub role: Option<String>,
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

        // Aplicar cambios básicos
        if let Some(name) = input.name {
            user.set_name(Some(name));
        }

        if let Some(active) = input.is_active {
            user.is_active = active;
        }

        // Guardar cambios en la entidad principal
        self.user_repo.save(&user).await?;

        // Actualizar roles si se especifica
        if let Some(new_role) = input.role {
            // Simplificación: removemos todos los roles y asignamos el nuevo
            // En un sistema real, un usuario podría tener múltiples roles
            let current_roles = ["admin", "user", "moderator", "superadmin"];
            for r in current_roles {
                let _ = self.user_repo.remove_role(id, r).await;
            }
            self.user_repo.assign_role(id, &new_role).await?;
        }

        Ok(user)
    }
}
