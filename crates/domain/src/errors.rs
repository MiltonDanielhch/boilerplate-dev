// Ubicación: `crates/domain/src/errors.rs`
//
// Descripción: Errores del dominio. Usa thiserror.
//
// ADRs relacionados: ADR 0007 (Errores)

use thiserror::Error;

/// Errores del dominio — no dependen de HTTP ni de infraestructura.
#[derive(Error, Debug, Clone)]
pub enum DomainError {
    // ─── Not Found ───────────────────────────────────────────────────────────
    #[error("Entidad no encontrada: {resource}")]
    NotFound { resource: String },

    // ─── Validación ──────────────────────────────────────────────────────────
    #[error("Validación fallida: {0}")]
    Validation(String),

    #[error("Email inválido: {reason}")]
    InvalidEmail { reason: String },

    #[error("Contraseña inválida: {reason}")]
    InvalidPassword { reason: String },

    #[error("Permiso inválido: {reason}")]
    InvalidPermission { reason: String },

    #[error("ID inválido: {message}")]
    InvalidId { message: String },

    // ─── Conflictos de negocio ───────────────────────────────────────────────
    #[error("Email ya existe: {email}")]
    EmailAlreadyExists { email: String },

    #[error("Credenciales inválidas")]
    InvalidCredentials,

    #[error("Token inválido o expirado")]
    InvalidToken,

    // ─── Autorización ────────────────────────────────────────────────────────
    #[error("Acceso denegado: {message}")]
    Forbidden { message: String },

    #[error("Permiso requerido: {permission}")]
    MissingPermission { permission: String },

    // ─── Dominio ─────────────────────────────────────────────────────────────
    #[error("Error de base de datos: {0}")]
    Database(String),

    #[error("Error interno: {0}")]
    Internal(String),
}

impl DomainError {
    /// Helper para crear NotFound genérico.
    pub fn not_found(resource: &str) -> Self {
        Self::NotFound {
            resource: resource.to_string(),
        }
    }

    /// Helper para crear Forbidden genérico.
    pub fn forbidden(message: &str) -> Self {
        Self::Forbidden {
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_not_found() {
        let err = DomainError::not_found("User");
        assert!(err.to_string().contains("User"));
    }

    #[test]
    fn display_forbidden() {
        let err = DomainError::forbidden("No admin");
        assert!(err.to_string().contains("No admin"));
    }
}

