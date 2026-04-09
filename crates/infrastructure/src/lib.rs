// Ubicación: `crates/infrastructure/src/lib.rs`
//
// Descripción: Adaptadores de infraestructura: HTTP, config, router.
//              Conoce Axum y todos los crates de aplicación.
//
// ADRs relacionados: ADR 0001, ADR 0003 (Axum)

pub mod http;
pub mod config;
pub mod router;
