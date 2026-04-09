// Ubicación: `crates/auth/src/lib.rs`
//
// Descripción: Autenticación con Argon2id + PASETO v4.
//              JWT está PROHIBIDO (ADR 0008).
//
// ADRs relacionados: ADR 0008 (PASETO), ADR 0001

pub mod password;
pub mod token;
