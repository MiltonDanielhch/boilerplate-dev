// Ubicación: `apps/api/src/middleware/mod.rs`
//
// Descripción: Módulos de middleware.

pub mod audit;
pub mod auth;
pub mod rbac;
// TODO: rate_limit — pendiente tower_governor ADR 0009
pub mod request_id;
pub mod trace;
