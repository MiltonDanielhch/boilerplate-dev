// Ubicación: `crates/application/src/content/list_content.rs`

use domain::entities::content_block::ContentBlock;
use domain::errors::DomainError;
use domain::ports::ContentRepository;

pub struct ListContentUseCase<R: ContentRepository> {
    content_repo: R,
}

impl<R: ContentRepository> ListContentUseCase<R> {
    pub fn new(content_repo: R) -> Self {
        Self { content_repo }
    }

    pub async fn execute(&self) -> Result<Vec<ContentBlock>, DomainError> {
        self.content_repo.list_blocks().await
    }
}
