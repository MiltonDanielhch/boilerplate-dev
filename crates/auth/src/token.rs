// Ubicación: `crates/auth/src/token.rs`
//
// Descripción: Opaque refresh tokens (32 bytes aleatorios) + hash SHA-256.
//              Los access tokens son PASETO v4 (implementados en paseto.rs).
//
// ADRs relacionados: ADR 0008

use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Genera un token opaco de 32 bytes (64 caracteres hex).
/// Usado para refresh tokens — no contienen datos, solo aleatoriedad.
pub fn generate_opaque_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Hashea un token para almacenamiento seguro en DB.
/// Usa SHA-256 — es determinista, permite buscar por hash.
pub fn hash_token(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_opaque_token() {
        let token = generate_opaque_token();
        assert_eq!(token.len(), 64); // 32 bytes = 64 hex chars
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_tokens_are_unique() {
        let token1 = generate_opaque_token();
        let token2 = generate_opaque_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_hash_token() {
        let raw = "my_secret_token";
        let hash1 = hash_token(raw);
        let hash2 = hash_token(raw);

        // Determinista: mismo input = mismo hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 = 32 bytes = 64 hex

        // Diferentes tokens = hashes diferentes
        let different_hash = hash_token("different_token");
        assert_ne!(hash1, different_hash);
    }
}
