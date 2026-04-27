// Ubicación: `crates/domain/src/entities/user.rs`
//
// Descripción: Entidad User con soporte para Soft Delete (ADR 0006).
//
// ADRs relacionados: ADR 0001, ADR 0006

use crate::value_objects::{Email, PasswordHash, UserId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Entidad User del dominio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub password_hash: PasswordHash,
    pub name: Option<String>,
    pub is_active: bool,
    pub email_verified_at: Option<OffsetDateTime>,
    pub last_login_at: Option<OffsetDateTime>,
    pub created_by: Option<UserId>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>, // Soft Delete (ADR 0006)
}

impl User {
    /// Crea un nuevo usuario (no verificado, activo por defecto).
    pub fn new(
        email: Email,
        password_hash: PasswordHash,
        name: Option<String>,
    ) -> Result<Self, crate::errors::DomainError> {
        let now = OffsetDateTime::now_utc();

        Ok(Self {
            id: UserId::new(),
            email,
            password_hash,
            name,
            is_active: true,
            email_verified_at: None,
            last_login_at: None,
            created_by: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
    }

    /// Verifica si el usuario está activo (no soft-deleted y flag activo).
    pub fn is_active(&self) -> bool {
        self.is_active && self.deleted_at.is_none()
    }

    /// Verifica si el email está verificado.
    pub fn is_email_verified(&self) -> bool {
        self.email_verified_at.is_some()
    }

    /// Marca el email como verificado.
    pub fn verify_email(&mut self) {
        self.email_verified_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Soft delete del usuario (ADR 0006).
    /// No elimina físicamente, marca deleted_at.
    pub fn soft_delete(&mut self) {
        self.deleted_at = Some(OffsetDateTime::now_utc());
        self.is_active = false;
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Verifica si está soft-deleted.
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Reactiva un usuario soft-deleted (con cuidado).
    pub fn reactivate(&mut self) {
        self.deleted_at = None;
        self.is_active = true;
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Cambia el nombre del usuario.
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
        self.updated_at = OffsetDateTime::now_utc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_email() -> Email {
        Email::new("test@example.com").unwrap()
    }

    fn valid_password() -> PasswordHash {
        PasswordHash::new("$argon2id$hash_placeholder").unwrap()
    }

    #[test]
    fn user_new_is_active_by_default() {
        let user = User::new(valid_email(), valid_password(), None).unwrap();
        assert!(user.is_active());
        assert!(!user.is_deleted());
    }

    #[test]
    fn email_not_verified_on_creation() {
        let user = User::new(valid_email(), valid_password(), None).unwrap();
        assert!(!user.is_email_verified());
    }

    #[test]
    fn verify_email_works() {
        let mut user = User::new(valid_email(), valid_password(), None).unwrap();
        user.verify_email();
        assert!(user.is_email_verified());
    }

    #[test]
    fn soft_delete_marks_deleted() {
        let mut user = User::new(valid_email(), valid_password(), None).unwrap();
        user.soft_delete();
        assert!(user.is_deleted());
        assert!(!user.is_active());
    }
}
