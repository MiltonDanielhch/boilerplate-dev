// Ubicación: `crates/domain/src/ports/mod.rs`
//
// Descripción: Puertos (traits) del dominio — interfaces que la capa de aplicación
//              e infraestructura deben implementar.
//
// ADRs relacionados: ADR 0001 (Hexagonal), ADR 0019 (Email), ADR 0020 (Storage)

pub mod audit_repository;
pub mod auth_provider;
pub mod lead_repository;
pub mod mailer;
pub mod session_repository;
pub mod storage_repository;
pub mod token_repository;
pub mod user_repository;

// Re-exports
pub use audit_repository::AuditRepository;
pub use auth_provider::{PasswordHasher, PasswordVerifier, TokenGenerator};
pub use lead_repository::LeadRepository;
pub use mailer::{EmailMessage, Mailer};
pub use session_repository::SessionRepository;
pub use storage_repository::{StorageObject, StorageRepository};
pub use token_repository::TokenRepository;
pub use user_repository::UserRepository;
