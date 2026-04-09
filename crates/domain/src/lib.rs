// Ubicación: `crates/domain/src/lib.rs`
//
// Descripción: Núcleo puro del dominio. Entidades, value objects, y reglas de negocio.
//              Sin dependencias de infraestructura (no sqlx, no axum).
//
// ADRs relacionados: ADR 0001 (Arquitectura Hexagonal)

pub mod entities;
pub mod value_objects;
pub mod errors;
