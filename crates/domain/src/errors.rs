// Ubicación: `crates/domain/src/errors.rs`
//
// Descripción: Errores del dominio. Usa thiserror.
//
// ADRs relacionados: ADR 0007 (Errores)

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Entidad no encontrada")]
    NotFound,
    #[error("Validación fallida: {0}")]
    Validation(String),
}
