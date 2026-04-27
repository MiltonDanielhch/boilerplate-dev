// Ubicación: `crates/application/src/content/update_content.rs`

use domain::entities::content_block::ContentBlock;
use domain::errors::DomainError;
use domain::ports::ContentRepository;
use domain::value_objects::UserId;
use time::OffsetDateTime;

pub struct UpdateContentInput {
    pub key: String,
    pub content: String,
    pub modified_by: UserId,
}

pub struct UpdateContentUseCase<R: ContentRepository> {
    content_repo: R,
}

impl<R: ContentRepository> UpdateContentUseCase<R> {
    pub fn new(content_repo: R) -> Self {
        Self { content_repo }
    }

    pub async fn execute(&self, input: UpdateContentInput) -> Result<(), DomainError> {
        let mut block = self.content_repo.get_block(&input.key).await?
            .ok_or_else(|| DomainError::not_found("Content block"))?;

        block.content = input.content;
        block.last_modified_by = Some(input.modified_by);
        block.updated_at = OffsetDateTime::now_utc();

        self.content_repo.save_block(&block).await
    }
}
