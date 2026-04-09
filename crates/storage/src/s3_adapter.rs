// Ubicación: `crates/storage/src/s3_adapter.rs`
//
// Descripción: Adaptador concreto de S3/Tigris.
//
// ADRs relacionados: ADR 0020

use crate::ports::StoragePort;

pub struct S3Adapter;

impl S3Adapter {
    pub fn new() -> Self {
        Self
    }
}

impl StoragePort for S3Adapter {
    async fn upload(&self, key: &str, _data: &[u8]) -> Result<String, String> {
        Ok(format!("s3://bucket/{}", key))
    }
}
