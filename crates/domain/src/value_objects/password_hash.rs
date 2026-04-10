// Ubicación: `crates/domain/src/value_objects/password_hash.rs`
//
// Descripción: Value object PasswordHash — nunca expone el hash.
//              Usa argon2id (ADR 0008).
//
// ADRs relacionados: ADR 0001, ADR 0008

use serde::{Deserialize, Serialize};

/// Value object PasswordHash.
/// Almacena el hash argon2id, nunca la contraseña en texto plano.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// Crea un nuevo PasswordHash desde string (ya hasheado).
    /// ⚠️ Este constructor NO hashea, solo almacena el hash existente.
    pub fn new(hash: &str) -> Result<Self, crate::errors::DomainError> {
        if hash.is_empty() {
            return Err(crate::errors::DomainError::InvalidPassword {
                reason: "Password hash cannot be empty".to_string(),
            });
        }

        // Verificación básica de formato argon2
        if !hash.starts_with("$argon2") {
            return Err(crate::errors::DomainError::InvalidPassword {
                reason: "Invalid argon2 hash format".to_string(),
            });
        }

        Ok(Self(hash.to_string()))
    }

    /// Verifica una contraseña en texto plano contra este hash.
    /// ⚠️ Este es un placeholder — la verificación real se hace en infraestructura.
    pub fn verify(&self, _password: &str) -> bool {
        // La verificación real usa argon2 y está en el crate auth/infrastructure
        // Este método es para la interfaz del dominio
        false // Placeholder
    }

    /// Retorna el hash (solo para persistencia).
    /// NUNCA expone esto en logs o APIs.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Formato seguro para Debug (truncado).
    pub fn format_secure(&self) -> String {
        format!("{}...", &self.0[..20.min(self.0.len())])
    }
}

impl std::fmt::Display for PasswordHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_argon2_hash() -> &'static str {
        "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$hash"
    }

    #[test]
    fn password_hash_valido_se_crea() {
        let hash = PasswordHash::new(valid_argon2_hash()).unwrap();
        assert!(hash.as_str().starts_with("$argon2id"));
    }

    #[test]
    fn password_hash_vacio_falla() {
        let result = PasswordHash::new("");
        assert!(result.is_err());
    }

    #[test]
    fn password_hash_sin_argon2_falla() {
        let result = PasswordHash::new("not_an_argon2_hash");
        assert!(result.is_err());
    }

    #[test]
    fn display_no_expone_hash() {
        let hash = PasswordHash::new(valid_argon2_hash()).unwrap();
        assert_eq!(hash.to_string(), "[REDACTED]");
    }
}
