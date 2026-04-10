// Ubicación: `crates/domain/src/value_objects/email.rs`
//
// Descripción: Value object Email — validado y normalizado a minúsculas.
//
// ADRs relacionados: ADR 0001

use serde::{Deserialize, Serialize};

/// Value object Email.
/// Inmutable, validado al construirse, normalizado a minúsculas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    /// Crea un nuevo Email validado.
    pub fn new(value: &str) -> Result<Self, crate::errors::DomainError> {
        let normalized = value.trim().to_lowercase();

        // Validación básica de formato email
        if normalized.is_empty() {
            return Err(crate::errors::DomainError::InvalidEmail {
                reason: "Email cannot be empty".to_string(),
            });
        }

        if !normalized.contains('@') {
            return Err(crate::errors::DomainError::InvalidEmail {
                reason: "Email must contain @".to_string(),
            });
        }

        let parts: Vec<&str> = normalized.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(crate::errors::DomainError::InvalidEmail {
                reason: "Invalid email format".to_string(),
            });
        }

        if !parts[1].contains('.') {
            return Err(crate::errors::DomainError::InvalidEmail {
                reason: "Domain must contain a dot".to_string(),
            });
        }

        Ok(Self(normalized))
    }

    /// Retorna el valor del email (minúsculas).
    pub fn value(&self) -> &str {
        &self.0
    }

    /// Retorna el dominio del email.
    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }

    /// Retorna la parte local (antes del @).
    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_valido_se_crea() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(email.value(), "test@example.com");
    }

    #[test]
    fn email_normalizado_a_minusculas() {
        let email = Email::new("TEST@EXAMPLE.COM").unwrap();
        assert_eq!(email.value(), "test@example.com");
    }

    #[test]
    fn email_con_espacios_se_trimmea() {
        let email = Email::new("  test@example.com  ").unwrap();
        assert_eq!(email.value(), "test@example.com");
    }

    #[test]
    fn email_sin_arroba_falla() {
        let result = Email::new("testexample.com");
        assert!(result.is_err());
    }

    #[test]
    fn email_vacio_falla() {
        let result = Email::new("");
        assert!(result.is_err());
    }

    #[test]
    fn email_domain_extrae_correctamente() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(email.domain(), "example.com");
    }
}
