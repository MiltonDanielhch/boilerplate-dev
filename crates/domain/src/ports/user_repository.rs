// Ubicación: `crates/domain/src/ports/user_repository.rs`
//
// Descripción: Puerto (trait) para persistencia de usuarios.
//              Implementado por la capa de infraestructura (SQLx).
//
// ADRs relacionados: ADR 0001 (Hexagonal), ADR 0005 (SQLx)

use crate::entities::User;
use crate::value_objects::{Email, UserId};
use crate::errors::DomainError;
use async_trait::async_trait;

/// Puerto para operaciones de persistencia de usuarios.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Busca usuario por ID.
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError>;

    /// Busca usuario activo por email (no soft-deleted).
    async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;

    /// Guarda (inserta o actualiza) un usuario.
    async fn save(&self, user: &User) -> Result<(), DomainError>;

    /// Soft delete de usuario.
    async fn soft_delete(&self, id: &UserId) -> Result<(), DomainError>;

    /// Verifica si un usuario tiene un permiso específico (vía roles).
    async fn has_permission(&self, user_id: &UserId, permission: &str) -> Result<bool, DomainError>;

    /// Lista usuarios con paginación.
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>, DomainError>;
}
