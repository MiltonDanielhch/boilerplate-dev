//! Ubicación: `crates/database/src/repositories/sqlite_session_repository.rs`
//!
//! Descripción: Implementación SQLite del repositorio de sesiones.
//!              Gestiona refresh tokens, revocación y expiración.
//!
//! ADRs relacionados: ADR 0008 (PASETO SSR), ADR 0004 (SQLite)

use domain::entities::Session;
use domain::errors::DomainError;
use domain::ports::SessionRepository;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Repositorio SQLite para sesiones y refresh tokens
#[derive(Debug, Clone)]
pub struct SqliteSessionRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteSessionRepository {
    /// Crear nueva instancia del repositorio
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

impl SessionRepository for SqliteSessionRepository {
    /// Crear nueva sesión
    async fn create(&self, session: &Session) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, refresh_token_hash, expires_at, is_revoked, ip_address, user_agent, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#
        )
        .bind(session.id.to_string())
        .bind(session.user_id.to_string())
        .bind(&session.refresh_token_hash)
        .bind(session.expires_at)
        .bind(session.is_revoked)
        .bind(&session.ip_address)
        .bind(&session.user_agent)
        .bind(session.created_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    /// Buscar sesión por hash del refresh token
    async fn find_by_token(&self, token_hash: &str) -> Result<Option<Session>, DomainError> {
        let row = sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT id, user_id, refresh_token_hash, expires_at, is_revoked, ip_address, user_agent, created_at, last_activity_at
            FROM sessions WHERE refresh_token_hash = ?1
            "#
        )
        .bind(token_hash)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(|r| r.into()))
    }

    /// Buscar todas las sesiones activas de un usuario
    async fn find_active_by_user(&self, user_id: &str) -> Result<Vec<Session>, DomainError> {
        let rows = sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT id, user_id, refresh_token_hash, expires_at, is_revoked, ip_address, user_agent, created_at, last_activity_at
            FROM sessions 
            WHERE user_id = ?1 AND is_revoked = FALSE AND expires_at > datetime('now')
            ORDER BY created_at DESC
            "#
        )
        .bind(user_id.to_string())
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Revocar una sesión específica
    async fn revoke(&self, id: &str) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE sessions 
            SET is_revoked = TRUE, revoked_at = datetime('now')
            WHERE id = ?1
            "#
        )
        .bind(id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    /// Revocar todas las sesiones de un usuario (logout global)
    async fn revoke_all_for_user(&self, user_id: &str) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE sessions 
            SET is_revoked = TRUE, revoked_at = datetime('now')
            WHERE user_id = ?1 AND is_revoked = FALSE
            "#
        )
        .bind(user_id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    /// Limpiar sesiones expiradas (maintenance)
    async fn cleanup_expired(&self, before: time::OffsetDateTime) -> Result<u64, DomainError> {
        let result = sqlx::query(
            r#"
            DELETE FROM sessions 
            WHERE expires_at < ?1 OR (is_revoked = TRUE AND revoked_at < ?1)
            "#
        )
        .bind(before)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Actualizar timestamp de última actividad
    async fn update_activity(&self, id: &str) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE sessions 
            SET last_activity_at = datetime('now')
            WHERE id = ?1
            "#
        )
        .bind(id.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }
}

/// Estructura intermedia para SQLx
#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    user_id: String,
    refresh_token_hash: String,
    expires_at: time::OffsetDateTime,
    is_revoked: bool,
    ip_address: Option<String>,
    user_agent: Option<String>,
    created_at: time::OffsetDateTime,
    last_activity_at: time::OffsetDateTime,
}

impl From<SessionRow> for Session {
    fn from(row: SessionRow) -> Self {
        Session {
            id: row.id,
            user_id: row.user_id,
            refresh_token_hash: row.refresh_token_hash,
            expires_at: row.expires_at,
            is_revoked: row.is_revoked,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            created_at: row.created_at,
            last_activity_at: row.last_activity_at,
        }
    }
}
