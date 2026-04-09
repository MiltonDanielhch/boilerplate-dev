// Ubicación: `crates/auth/src/password.rs`
//
// Descripción: Hash de contraseñas con Argon2id.
//
// ADRs relacionados: ADR 0008

pub fn hash_password(password: &str) -> String {
    // TODO: Implementar con argon2
    format!("hashed_{}", password)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    hash == format!("hashed_{}", password)
}
