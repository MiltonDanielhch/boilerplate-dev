// Ubicación: `crates/domain/src/ports/storage_repository.rs`
//
// Descripción: Puerto (trait) para almacenamiento de archivos.
//              Implementado por adaptadores (S3/Tigris, local filesystem, etc.).
//
// ADRs relacionados: ADR 0001, ADR 0020

use crate::errors::DomainError;
use async_trait::async_trait;

/// Metadatos de un objeto almacenado.
#[derive(Debug, Clone)]
pub struct StorageObject {
    pub key: String,
    pub size: u64,
    pub content_type: Option<String>,
    pub etag: Option<String>,
}

/// Puerto para operaciones de almacenamiento.
#[async_trait]
pub trait StorageRepository: Send + Sync {
    /// Almacena un objeto.
    async fn put(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: Option<&str>,
    ) -> Result<StorageObject, DomainError>;

    /// Recupera un objeto.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, DomainError>;

    /// Elimina un objeto.
    async fn delete(&self, key: &str) -> Result<(), DomainError>;

    /// Verifica si existe.
    async fn exists(&self, key: &str) -> Result<bool, DomainError>;

    /// Genera URL prefirmada para descarga (si soportado).
    async fn get_presigned_url(&self, key: &str, expires_in_secs: u64) -> Result<String, DomainError>;
}
