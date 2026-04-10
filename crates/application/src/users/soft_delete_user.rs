// Ubicación: `crates/application/src/users/soft_delete_user.rs`
//
// Descripción: Caso de uso Soft Delete de Usuario.
//              UPDATE deleted_at — NUNCA DELETE real (ADR 0006).
//
// ADRs relacionados: ADR 0001, ADR 0006 (Soft Delete)

use domain::errors::DomainError;
use domain::ports::UserRepository;
use domain::value_objects::UserId;

/// Caso de uso: Soft delete de usuario.
pub struct SoftDeleteUserUseCase<R: UserRepository> {
    user_repo: R,
}

impl<R: UserRepository> SoftDeleteUserUseCase<R> {
    pub fn new(user_repo: R) -> Self {
        Self { user_repo }
    }

    /// Realiza soft delete del usuario.
    /// ⚠️ NUNCA hace DELETE físico — solo UPDATE deleted_at.
    pub async fn execute(&self, id: &UserId) -> Result<(), DomainError> {
        // Verificar que usuario existe y está activo
        match self.user_repo.find_by_id(id).await? {
            Some(user) if user.is_active() => {
                self.user_repo.soft_delete(id).await
            }
            _ => Err(DomainError::not_found("User")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Agregar tests con mockall
    // - soft_delete_marca_usuario_como_inactivo()
    // - soft_delete_de_usuario_inexistente_falla()
    // - soft_delete_no_borra_fisicamente_de_db()
}
