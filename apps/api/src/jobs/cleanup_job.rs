// Ubicación: `apps/api/src/jobs/cleanup_job.rs`
//
// Descripción: Job de limpieza para eliminar datos viejos.
//
// ADRs relacionados: ADR 0018

use apalis::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupJob {
    pub task: CleanupTask,
    #[sqlx(skip)]
    pub pool: Arc<SqlitePool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupTask {
    CleanupExpiredTokens,
    CleanupRevokedSessions,
    CleanupOldAuditLogs,
    FullCleanup,
}

impl CleanupJob {
    pub fn new(task: CleanupTask, pool: Arc<SqlitePool>) -> Self {
        Self { task, pool }
    }
}

#[async_trait::async_trait]
impl Job for CleanupJob {
    async fn run(&self, _ctx: JobContext) -> Result<(), JobError> {
        let pool = self.pool.clone();
        
        match self.task {
            CleanupTask::CleanupExpiredTokens => {
                let result = sqlx::query(
                    r#"DELETE FROM tokens WHERE expires_at < datetime('now')"#
                )
                .execute(&*pool)
                .await
                .map_err(|e| JobError::Unexpected(e.to_string()))?;
                
                tracing::info!(task = "cleanup_expired_tokens", deleted = result.rows_affected(), "Cleanup job completed");
            }
            CleanupTask::CleanupRevokedSessions => {
                let result = sqlx::query(
                    r#"DELETE FROM sessions WHERE is_revoked = TRUE AND expires_at < datetime('now')"#
                )
                .execute(&*pool)
                .await
                .map_err(|e| JobError::Unexpected(e.to_string()))?;
                
                tracing::info!(task = "cleanup_revoked_sessions", deleted = result.rows_affected(), "Cleanup job completed");
            }
            CleanupTask::CleanupOldAuditLogs => {
                let result = sqlx::query(
                    r#"DELETE FROM audit_logs WHERE created_at < datetime('now', '-30 days')"#
                )
                .execute(&*pool)
                .await
                .map_err(|e| JobError::Unexpected(e.to_string()))?;
                
                tracing::info!(task = "cleanup_old_audit_logs", deleted = result.rows_affected(), "Cleanup job completed");
            }
            CleanupTask::FullCleanup => {
                let tokens = sqlx::query(r#"DELETE FROM tokens WHERE expires_at < datetime('now')"#)
                    .execute(&*pool)
                    .await
                    .map_err(|e| JobError::Unexpected(e.to_string()))?
                    .rows_affected();
                
                let sessions = sqlx::query(r#"DELETE FROM sessions WHERE is_revoked = TRUE AND expires_at < datetime('now')"#)
                    .execute(&*pool)
                    .await
                    .map_err(|e| JobError::Unexpected(e.to_string()))?
                    .rows_affected();
                
                let audit = sqlx::query(r#"DELETE FROM audit_logs WHERE created_at < datetime('now', '-30 days')"#)
                    .execute(&*pool)
                    .await
                    .map_err(|e| JobError::Unexpected(e.to_string()))?
                    .rows_affected();
                
                tracing::info!(task = "full_cleanup", tokens, sessions, audit, "Full cleanup completed");
            }
        }

        Ok(())
    }
}