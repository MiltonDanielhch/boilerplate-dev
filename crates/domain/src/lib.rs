// Ubicación: `crates/domain/src/lib.rs`
//
// Descripción: Núcleo puro del dominio. Entidades, value objects, puertos y reglas de negocio.
//              Sin dependencias de infraestructura (no sqlx, no axum).
//
// ADRs relacionados: ADR 0001 (Arquitectura Hexagonal)

pub mod entities;
pub mod errors;
pub mod ports;
pub mod value_objects;

// Re-exports convenientes
pub use entities::{user::User, role::Role, session::Session, audit_log::AuditLog, lead::Lead};
pub use value_objects::{email::Email, user_id::UserId, password_hash::PasswordHash, permission::Permission};
