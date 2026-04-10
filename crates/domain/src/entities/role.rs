// Ubicación: `crates/domain/src/entities/role.rs`
//
// Descripción: Entidad Role para RBAC (ADR 0006).
//
// ADRs relacionados: ADR 0001, ADR 0006

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Entidad Role del dominio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String, // UUID v7
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool, // No se puede eliminar
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>, // Soft Delete
}

impl Role {
    /// Crea un nuevo rol.
    pub fn new(name: String, description: Option<String>, is_system: bool) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::now_v7().to_string(),
            name,
            description,
            is_system,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Verifica si el rol está activo.
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    /// Soft delete del rol (solo si no es sistema).
    pub fn soft_delete(&mut self) -> Result<(), crate::errors::DomainError> {
        if self.is_system {
            return Err(crate::errors::DomainError::Forbidden {
                message: "Cannot delete system role".to_string(),
            });
        }
        self.deleted_at = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
        Ok(())
    }
}
