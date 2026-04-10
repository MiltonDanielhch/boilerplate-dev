// Ubicación: `crates/domain/src/ports/storage_repository.rs`
//
// Descripción: Puerto (trait) para almacenamiento de archivos.
//              Implementado por adaptadores (S3/Tigris, local filesystem, etc.).
//
// ADRs relacionados: ADR 0001, ADR 0020

use crate::errors::DomainError;
use std::future::Future;

/// Metadatos de un objeto almacenado.
#[derive(Debug, Clone)]
pub struct StorageObject {
    pub key: String,
    pub size: u64,
    pub content_type: Option<String>,
    pub etag: Option<String>,
}

/// Puerto para operaciones de almacenamiento.
pub trait StorageRepository: Send + Sync {
    /// Almacena un objeto.
    fn put(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: Option<&str>,
    ) -> impl Future<Output = Result<StorageObject, DomainError>> + Send;

    /// Recupera un objeto.
    fn get(&self, key: &str) -> impl Future<Output = Result<Option<Vec<u8>>, DomainError>> + Send;

    /// Elimina un objeto.
    fn delete(&self, key: &str) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Verifica si existe.
    fn exists(&self, key: &str) -> impl Future<Output = Result<bool, DomainError>> + Send;

    /// Genera URL prefirmada para descarga (si soportado).
    fn get_presigned_url(&self, key: &str, expires_in_secs: u64) -> impl Future<Output = Result<String, DomainError>> + Send;
}
