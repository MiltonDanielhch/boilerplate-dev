// Ubicación: `crates/application/src/users/impersonate_user.rs`
//
// Descripción: Caso de uso Impersonate User.
//              Permite a un admin generar un token válido para otro usuario.
//
// ADRs relacionados: ADR 0006 (RBAC), ADR 0008 (PASETO)

use domain::errors::DomainError;
use domain::ports::{UserRepository, TokenGenerator};
use domain::value_objects::UserId;
use tracing::warn;
use std::sync::Arc;

pub struct ImpersonateUserInput {
    pub target_user_id: UserId,
    pub admin_id: UserId,
}

pub struct ImpersonateUserUseCase<R: UserRepository, T: TokenGenerator> {
    user_repo: R,
    token_gen: Arc<T>,
}

impl<R: UserRepository, T: TokenGenerator> ImpersonateUserUseCase<R, T> {
    pub fn new(user_repo: R, token_gen: Arc<T>) -> Self {
        Self { user_repo, token_gen }
    }

    pub async fn execute(&self, input: ImpersonateUserInput) -> Result<String, DomainError> {
        // 1. Verificar que el admin es realmente admin
        let is_admin = self.user_repo.has_role(&input.admin_id, "admin").await?;
        let is_super = self.user_repo.has_role(&input.admin_id, "superadmin").await?;
        
        if !is_admin && !is_super {
            return Err(DomainError::forbidden("Only admins can impersonate users"));
        }

        // 2. Verificar que el usuario objetivo existe
        let user = self.user_repo.find_by_id(&input.target_user_id).await?
            .ok_or_else(|| DomainError::not_found("Target User"))?;

        if !user.is_active() {
            return Err(DomainError::Validation("Cannot impersonate inactive user".to_string()));
        }

        // 3. Generar token PASETO para el usuario objetivo
        // Usamos uuid::Uuid directamente si el puerto lo pide
        let token = self.token_gen.generate_access_token(&input.target_user_id.uuid())?;

        // 4. Loggear la acción (TODO: Implementar Audit Logs detallados)
        warn!(
            admin_id = %input.admin_id, 
            target_id = %input.target_user_id, 
            "User impersonation started"
        );

        Ok(token)
    }
}
