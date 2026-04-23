// Ubicación: `crates/database/src/repositories/sqlite_audit_repository.rs`
//
// Descripción: Implementación SQLite de AuditRepository.
//              Insert-only, nunca se modifica ni elimina.
//
// ADRs relacionados: ADR 0001, ADR 0004, ADR 0006

use crate::models::audit_row::AuditRow;
use domain::entities::AuditLog;
use domain::errors::DomainError;
use domain::ports::AuditRepository;
use sqlx::{Pool, Sqlite};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct SqliteAuditRepository {
    pool: Pool<Sqlite>,
}

impl SqliteAuditRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

impl AuditRepository for SqliteAuditRepository {
    async fn log(&self, log: &AuditLog) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (id, user_id, action, resource, resource_id,
                                   ip_address, user_agent, details, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&log.id)
        .bind(&log.user_id)
        .bind(&log.action)
        .bind(&log.resource)
        .bind(&log.resource_id)
        .bind(&log.ip_address)
        .bind(&log.user_agent)
        .bind(&log.details)
        .bind(log.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_by_resource(
        &self,
        resource: &str,
        resource_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<AuditLog>, DomainError> {
        let rows = match resource_id {
            Some(rid) => {
                sqlx::query_as::<_, AuditRow>(
                    r#"
                    SELECT id, user_id, action, resource, resource_id,
                           ip_address, user_agent, details, created_at
                    FROM audit_logs
                    WHERE resource = ?1 AND resource_id = ?2
                    ORDER BY created_at DESC
                    LIMIT ?3
                    "#
                )
                .bind(resource)
                .bind(rid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, AuditRow>(
                    r#"
                    SELECT id, user_id, action, resource, resource_id,
                           ip_address, user_agent, details, created_at
                    FROM audit_logs
                    WHERE resource = ?1
                    ORDER BY created_at DESC
                    LIMIT ?2
                    "#
                )
                .bind(resource)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| r.into_audit_log())
            .collect()
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<AuditLog>, DomainError> {
        let rows = sqlx::query_as::<_, AuditRow>(
            r#"
            SELECT id, user_id, action, resource, resource_id,
                   ip_address, user_agent, details, created_at
            FROM audit_logs
            WHERE user_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| r.into_audit_log())
            .collect()
    }

    async fn find_by_date_range(
        &self,
        from: OffsetDateTime,
        to: OffsetDateTime,
        limit: i64,
    ) -> Result<Vec<AuditLog>, DomainError> {
        let rows = sqlx::query_as::<_, AuditRow>(
            r#"
            SELECT id, user_id, action, resource, resource_id,
                   ip_address, user_agent, details, created_at
            FROM audit_logs
            WHERE created_at >= ?1 AND created_at <= ?2
            ORDER BY created_at DESC
            LIMIT ?3
            "#
        )
        .bind(from)
        .bind(to)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| r.into_audit_log())
            .collect()
    }
}