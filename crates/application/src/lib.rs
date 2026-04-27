// Ubicación: `crates/application/src/lib.rs`
//
// Descripción: Casos de uso (Use Cases). Coordina el dominio.
//
// ADRs relacionados: ADR 0001

pub mod admin;
pub mod audit;
pub mod auth;
pub mod content;
pub mod leads;
pub mod settings;
pub mod users;

// Legacy module (preservado temporalmente)
pub mod use_cases;
