// Ubicación: `crates/domain/src/ports/user_repository.rs`
//
// Descripción: Puerto (trait) para persistencia de usuarios.
//              Implementado por la capa de infraestructura (SQLx).
//
// ADRs relacionados: ADR 0001 (Hexagonal), ADR 0005 (SQLx)

use crate::entities::User;
use crate::value_objects::{Email, UserId};
use crate::errors::DomainError;
use std::future::Future;

/// Puerto para operaciones de persistencia de usuarios.
pub trait UserRepository: Send + Sync {
    /// Busca usuario por ID.
    fn find_by_id(&self, id: &UserId) -> impl Future<Output = Result<Option<User>, DomainError>> + Send;

    /// Busca usuario activo por email (no soft-deleted).
    fn find_active_by_email(&self, email: &Email) -> impl Future<Output = Result<Option<User>, DomainError>> + Send;

    /// Guarda (inserta o actualiza) un usuario.
    fn save(&self, user: &User) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Soft delete de usuario.
    fn soft_delete(&self, id: &UserId) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Verifica si un usuario tiene un permiso específico (vía roles).
    fn has_permission(&self, user_id: &UserId, permission: &str) -> impl Future<Output = Result<bool, DomainError>> + Send;

    /// Obtiene todos los permisos de un usuario (vía roles).
    fn get_permissions(&self, user_id: &UserId) -> impl Future<Output = Result<Vec<String>, DomainError>> + Send;

    /// Lista usuarios con paginación.
    fn list(&self, limit: i64, offset: i64) -> impl Future<Output = Result<Vec<User>, DomainError>> + Send;
}
