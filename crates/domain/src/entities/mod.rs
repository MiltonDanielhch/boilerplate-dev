// Ubicación: `crates/domain/src/entities/mod.rs`
//
// Descripción: Módulo de entidades del dominio.
//              Cada entidad encapsula estado y comportamiento de negocio.
//
// ADRs relacionados: ADR 0001, ADR 0006 (Soft Delete)

pub mod audit_log;
pub mod content_block;
pub mod lead;
pub mod role;
pub mod session;
pub mod system_setting;
pub mod user;

// Re-exports
pub use audit_log::AuditLog;
pub use lead::Lead;
pub use role::Role;
pub use session::Session;
pub use user::User;
