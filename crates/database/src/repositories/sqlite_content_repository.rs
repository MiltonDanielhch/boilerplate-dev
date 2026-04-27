// Ubicación: `crates/database/src/repositories/sqlite_content_repository.rs`

use domain::entities::content_block::ContentBlock;
use domain::errors::DomainError;
use domain::ports::ContentRepository;
use domain::value_objects::UserId;
use sqlx::{SqlitePool, FromRow};
use std::sync::Arc;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct SqliteContentRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteContentRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct ContentBlockRow {
    key: String,
    content: String,
    content_type: String,
    last_modified_by: Option<String>,
    updated_at: OffsetDateTime,
}

impl ContentBlockRow {
    fn into_entity(self) -> Result<ContentBlock, DomainError> {
        let last_modified_by = match self.last_modified_by {
            Some(id) => Some(UserId::parse(&id).map_err(|_| DomainError::Internal("Invalid UUID in content_blocks".to_string()))?),
            None => None,
        };

        Ok(ContentBlock {
            key: self.key,
            content: self.content,
            content_type: self.content_type,
            last_modified_by,
            updated_at: self.updated_at,
        })
    }
}

impl ContentRepository for SqliteContentRepository {
    async fn get_block(&self, key: &str) -> Result<Option<ContentBlock>, DomainError> {
        let row = sqlx::query_as::<_, ContentBlockRow>(
            "SELECT * FROM content_blocks WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.into_entity()?)),
            None => Ok(None),
        }
    }

    async fn save_block(&self, block: &ContentBlock) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO content_blocks (key, content, content_type, last_modified_by, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                content = excluded.content,
                content_type = excluded.content_type,
                last_modified_by = excluded.last_modified_by,
                updated_at = excluded.updated_at
            "#
        )
        .bind(&block.key)
        .bind(&block.content)
        .bind(&block.content_type)
        .bind(block.last_modified_by.as_ref().map(|id| id.to_string()))
        .bind(block.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_blocks(&self) -> Result<Vec<ContentBlock>, DomainError> {
        let rows = sqlx::query_as::<_, ContentBlockRow>(
            "SELECT * FROM content_blocks ORDER BY key ASC"
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| r.into_entity())
            .collect()
    }
}
