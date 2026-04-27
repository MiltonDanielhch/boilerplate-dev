// Ubicación: `crates/domain/src/ports/content_repository.rs`

use crate::entities::content_block::ContentBlock;
use crate::errors::DomainError;
use std::future::Future;

pub trait ContentRepository: Send + Sync {
    fn get_block(&self, key: &str) -> impl Future<Output = Result<Option<ContentBlock>, DomainError>> + Send;
    fn save_block(&self, block: &ContentBlock) -> impl Future<Output = Result<(), DomainError>> + Send;
    fn list_blocks(&self) -> impl Future<Output = Result<Vec<ContentBlock>, DomainError>> + Send;
}
