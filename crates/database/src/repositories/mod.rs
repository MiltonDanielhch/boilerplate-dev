// Ubicación: `crates/database/src/repositories/mod.rs`
//
// Descripción: Implementaciones de puertos (traits) del dominio.
//              Repositorios SQLx + decoradores de caché.
//
// ADRs relacionados: ADR 0004, ADR 0017

pub mod sqlite_lead_repository;
pub mod sqlite_session_repository;
pub mod sqlite_user_repository;

pub use sqlite_lead_repository::SqliteLeadRepository;
pub use sqlite_session_repository::SqliteSessionRepository;
pub use sqlite_user_repository::SqliteUserRepository;
