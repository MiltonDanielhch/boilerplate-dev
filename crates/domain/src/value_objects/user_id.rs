// Ubicación: `crates/domain/src/value_objects/user_id.rs`
//
// Descripción: Value object UserId — newtype sobre Uuid (v7).
//
// ADRs relacionados: ADR 0001

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Value object UserId — identificador único de usuario.
/// Usa UUID v7 (ordenado temporalmente).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UserId(Uuid);

impl UserId {
    /// Genera un nuevo UserId (UUID v7).
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Crea desde un Uuid existente.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Crea desde string.
    pub fn parse(s: &str) -> Result<Self, crate::errors::DomainError> {
        let uuid = Uuid::parse_str(s).map_err(|_| crate::errors::DomainError::InvalidId {
            message: format!("Invalid UUID: {}", s),
        })?;
        Ok(Self(uuid))
    }

    /// Retorna el Uuid interno.
    pub fn uuid(&self) -> Uuid {
        self.0
    }

    /// Retorna como string.
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_id_new_genera_uuid() {
        let id = UserId::new();
        assert!(!id.to_string().is_empty());
    }

    #[test]
    fn user_id_parse_valido() {
        let uuid_str = "018f1b4e-5b4e-7c3e-8f1b-4e5b4e7c3e8f";
        let id = UserId::parse(uuid_str).unwrap();
        assert_eq!(id.to_string(), uuid_str);
    }

    #[test]
    fn user_id_parse_invalido_falla() {
        let result = UserId::parse("not-a-uuid");
        assert!(result.is_err());
    }
}
