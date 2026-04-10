// Ubicación: `crates/domain/src/value_objects/permission.rs`
//
// Descripción: Value object Permission — formato "recurso:acción".
//              Ejemplos: "users:read", "users:write", "audit:read"
//
// ADRs relacionados: ADR 0001, ADR 0006

use serde::{Deserialize, Serialize};

/// Value object Permission.
/// Formato estándar: "recurso:acción".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Permission {
    resource: String,
    action: String,
}

impl Permission {
    /// Crea un nuevo Permission.
    pub fn new(resource: &str, action: &str) -> Result<Self, crate::errors::DomainError> {
        let resource = resource.trim().to_lowercase();
        let action = action.trim().to_lowercase();

        if resource.is_empty() || action.is_empty() {
            return Err(crate::errors::DomainError::InvalidPermission {
                reason: "Resource and action cannot be empty".to_string(),
            });
        }

        // Validar caracteres permitidos
        let valid_chars = |s: &str| s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-');
        if !valid_chars(&resource) || !valid_chars(&action) {
            return Err(crate::errors::DomainError::InvalidPermission {
                reason: "Invalid characters in resource or action".to_string(),
            });
        }

        Ok(Self { resource, action })
    }

    /// Parse desde string formato "recurso:acción".
    pub fn parse(s: &str) -> Result<Self, crate::errors::DomainError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(crate::errors::DomainError::InvalidPermission {
                reason: "Permission must be in format 'resource:action'".to_string(),
            });
        }
        Self::new(parts[0], parts[1])
    }

    /// Retorna el recurso.
    pub fn resource(&self) -> &str {
        &self.resource
    }

    /// Retorna la acción.
    pub fn action(&self) -> &str {
        &self.action
    }

    /// Verifica si este permiso implica otro (mismo recurso, acción más amplia).
    /// Por ejemplo, "users:write" implica "users:read".
    pub fn implies(&self, other: &Permission) -> bool {
        if self.resource != other.resource {
            return false;
        }

        // write implica read, delete implica write y read
        match (self.action.as_str(), other.action.as_str()) {
            ("delete", _) => true, // delete implica todo
            ("write", "read") => true,
            ("write", "write") => true,
            ("read", "read") => true,
            (a, b) => a == b,
        }
    }

    /// Lista de permisos del sistema.
    pub fn system_permissions() -> Vec<Permission> {
        vec![
            Self::new("users", "read").unwrap(),
            Self::new("users", "write").unwrap(),
            Self::new("users", "delete").unwrap(),
            Self::new("roles", "read").unwrap(),
            Self::new("roles", "write").unwrap(),
            Self::new("audit", "read").unwrap(),
        ]
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.resource, self.action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_new_valido() {
        let p = Permission::new("users", "read").unwrap();
        assert_eq!(p.resource(), "users");
        assert_eq!(p.action(), "read");
    }

    #[test]
    fn permission_parse_valido() {
        let p = Permission::parse("users:write").unwrap();
        assert_eq!(p.to_string(), "users:write");
    }

    #[test]
    fn permission_parse_invalido_falla() {
        let result = Permission::parse("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn permission_implies_funciona() {
        let write = Permission::parse("users:write").unwrap();
        let read = Permission::parse("users:read").unwrap();
        let delete = Permission::parse("users:delete").unwrap();

        assert!(write.implies(&read));  // write implica read
        assert!(delete.implies(&write)); // delete implica write
        assert!(delete.implies(&read)); // delete implica read
        assert!(!read.implies(&write)); // read NO implica write
    }

    #[test]
    fn permission_diferente_recurso_no_implica() {
        let users_write = Permission::parse("users:write").unwrap();
        let roles_read = Permission::parse("roles:read").unwrap();

        assert!(!users_write.implies(&roles_read));
    }
}
