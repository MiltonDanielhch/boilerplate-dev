// Ubicación: `crates/domain/src/value_objects/mod.rs`
//
// Descripción: Value objects del dominio. Inmutables, validados al construirse,
//              iguales si sus valores son iguales (identidad basada en valor).
//
// ADRs relacionados: ADR 0001, ADR 0006

pub mod email;
pub mod password_hash;
pub mod permission;
pub mod user_id;

// Re-exports
pub use email::Email;
pub use password_hash::PasswordHash;
pub use permission::Permission;
pub use user_id::UserId;
