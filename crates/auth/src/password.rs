// Ubicación: `crates/auth/src/password.rs`
//
// Descripción: Hash de contraseñas con Argon2id (OWASP 2024).
//              Parámetros: m=19456, t=2, p=1 (mínimo recomendado)
//
// ADRs relacionados: ADR 0008

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use domain::errors::DomainError;
use rand::rngs::OsRng;

/// Hashea una contraseña usando Argon2id.
/// Parámetros OWASP 2024: m=19456 KiB, t=2 iteraciones, p=1 hilo
pub fn hash_password(password: &str) -> Result<String, DomainError> {
    // Parámetros OWASP 2024 mínimos recomendados
    let params = Params::new(19456, 2, 1, None)
        .map_err(|e| DomainError::Internal(format!("Invalid argon2 params: {}", e)))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| DomainError::Internal(format!("Password hashing failed: {}", e)))?;

    Ok(password_hash.to_string())
}

/// Verifica una contraseña contra un hash.
/// Usa comparación en tiempo constante para prevenir timing attacks.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, DomainError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| DomainError::InvalidPassword {
            reason: format!("Invalid password hash format: {}", e),
        })?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub struct Argon2Verifier;

impl domain::ports::PasswordVerifier for Argon2Verifier {
    fn verify_password(&self, plain: &str, hash: &str) -> Result<bool, DomainError> {
        verify_password(plain, hash)
    }
}

impl domain::ports::PasswordHasher for Argon2Verifier {
    fn hash_password(&self, plain: &str) -> Result<String, DomainError> {
        hash_password(plain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "12345678";
        let hash = hash_password(password).expect("Should hash password");
        println!("Hash generado: {}", hash);

        // Verificación correcta
        assert!(verify_password(password, &hash).expect("Should verify"));

        // Verificación incorrecta
        assert!(!verify_password("wrong_password", &hash).expect("Should fail verification"));
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let password = "test_password";
        let hash1 = hash_password(password).expect("Should hash");
        let hash2 = hash_password(password).expect("Should hash again");

        // Salt aleatorio produce hashes diferentes
        assert_ne!(hash1, hash2);

        // Ambos deben verificar correctamente
        assert!(verify_password(password, &hash1).expect("Should verify hash1"));
        assert!(verify_password(password, &hash2).expect("Should verify hash2"));
    }
}
