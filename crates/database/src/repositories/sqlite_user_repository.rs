// Ubicación: `crates/database/src/repositories/sqlite_user_repository.rs`
//
// Descripción: Implementación de UserRepository con SQLx (SQLite).
//              ÚNICO lugar con queries SQL en la codebase.
//
// ADRs relacionados: ADR 0004, ADR 0001, ADR 0006 (Soft Delete)

use domain::entities::User;
use domain::errors::DomainError;
use domain::ports::UserRepository;
use domain::value_objects::{Email, UserId};
use sqlx::{Pool, Sqlite};

use crate::models::user_row::UserRow;

/// Implementación SQLite de UserRepository.
#[derive(Debug, Clone)]
pub struct SqliteUserRepository {
    pool: Pool<Sqlite>,
}

impl SqliteUserRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Convierte UserRow a entidad del dominio.
    fn row_to_entity(&self, row: UserRow) -> Result<User, DomainError> {
        let email = Email::new(&row.email).map_err(|_| DomainError::Internal(
            format!("Email corrupto en DB: {}", row.email)
        ))?;

        let password_hash = domain::value_objects::PasswordHash::new(&row.password_hash)
            .map_err(|_| DomainError::Internal(
                "Password hash corrupto en DB".to_string()
            ))?;

        let id = UserId::parse(&row.id).map_err(|_| DomainError::Internal(
            format!("UUID corrupto en DB: {}", row.id)
        ))?;

        // Construir User manualmente (no usamos User::new porque ya existe)
        // Usamos serde o manual construction
        Ok(User {
            id,
            email,
            password_hash,
            name: row.name,
            is_active: row.is_active,
            email_verified_at: row.email_verified_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        })
    }
}

impl UserRepository for SqliteUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, email, password_hash, name, is_active,
                   email_verified_at, created_at, updated_at, deleted_at
            FROM users
            WHERE id = ?
            "#
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(self.row_to_entity(r)?)),
            None => Ok(None),
        }
    }

    async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        // Usa índice parcial idx_users_email_active (WHERE deleted_at IS NULL)
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, email, password_hash, name, is_active,
                   email_verified_at, created_at, updated_at, deleted_at
            FROM users
            WHERE email = ?
              AND deleted_at IS NULL
            "#
        )
        .bind(email.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) if r.is_active() => Ok(Some(self.row_to_entity(r)?)),
            _ => Ok(None),
        }
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, name, is_active,
                             email_verified_at, created_at, updated_at, deleted_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                email = excluded.email,
                password_hash = excluded.password_hash,
                name = excluded.name,
                is_active = excluded.is_active,
                email_verified_at = excluded.email_verified_at,
                updated_at = excluded.updated_at,
                deleted_at = excluded.deleted_at
            "#
        )
        .bind(user.id.to_string())
        .bind(user.email.value())
        .bind(user.password_hash.as_str())
        .bind(&user.name)
        .bind(user.is_active)
        .bind(user.email_verified_at)
        .bind(user.created_at)
        .bind(user.updated_at)
        .bind(user.deleted_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn soft_delete(&self, id: &UserId) -> Result<(), DomainError> {
        // Soft Delete: UPDATE deleted_at — NUNCA DELETE real (ADR 0006)
        let now = time::OffsetDateTime::now_utc();

        sqlx::query(
            r#"
            UPDATE users
            SET deleted_at = ?,
                is_active = FALSE,
                updated_at = ?
            WHERE id = ?
              AND deleted_at IS NULL
            "#
        )
        .bind(now)
        .bind(now)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn has_permission(&self, user_id: &UserId, permission: &str) -> Result<bool, DomainError> {
        // Parse permission format "resource:action"
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() != 2 {
            return Err(DomainError::InvalidPermission {
                reason: "Formato debe ser 'resource:action'".to_string(),
            });
        }
        let resource = parts[0];
        let action = parts[1];

        // JOIN 4 tablas: users → user_roles → roles → role_permissions → permissions
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) as count
            FROM users u
            JOIN user_roles ur ON ur.user_id = u.id
            JOIN roles r ON r.id = ur.role_id AND r.deleted_at IS NULL
            JOIN role_permissions rp ON rp.role_id = r.id
            JOIN permissions p ON p.id = rp.permission_id
            WHERE u.id = ?
              AND u.deleted_at IS NULL
              AND p.resource = ?
              AND p.action = ?
            "#
        )
        .bind(user_id.to_string())
        .bind(resource)
        .bind(action)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(count > 0)
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>, DomainError> {
        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, email, password_hash, name, is_active,
                   email_verified_at, created_at, updated_at, deleted_at
            FROM users
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| self.row_to_entity(r))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::create_pool;

    async fn setup_test_db() -> Pool<Sqlite> {
        // Usa :memory: para tests de integración
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Falló conexión :memory:");

        // Crear tabla para tests
        sqlx::query(
            r#"
            CREATE TABLE users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                password_hash TEXT NOT NULL,
                name TEXT,
                is_active BOOLEAN DEFAULT TRUE,
                email_verified_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                deleted_at TIMESTAMPTZ
            )
            "#
        )
        .execute(&pool)
        .await
        .expect("Falló CREATE TABLE");

        pool
    }

    fn valid_email() -> Email {
        Email::new("test@example.com").unwrap()
    }

    fn valid_hash() -> domain::value_objects::PasswordHash {
        domain::value_objects::PasswordHash::new(
            "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$hash"
        ).unwrap()
    }

    #[tokio::test]
    async fn guardar_y_recuperar_usuario() {
        let pool = setup_test_db().await;
        let repo = SqliteUserRepository::new(pool);

        let user = User::new(valid_email(), valid_hash(), Some("Test".to_string())).unwrap();
        let id = user.id.clone();

        repo.save(&user).await.unwrap();
        let found = repo.find_by_id(&id).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().email.value(), "test@example.com");
    }

    #[tokio::test]
    async fn soft_delete_oculta_el_usuario() {
        let pool = setup_test_db().await;
        let repo = SqliteUserRepository::new(pool);

        let user = User::new(valid_email(), valid_hash(), None).unwrap();
        let id = user.id.clone();

        repo.save(&user).await.unwrap();
        repo.soft_delete(&id).await.unwrap();

        let found = repo.find_by_id(&id).await.unwrap();
        assert!(found.is_none() || !found.unwrap().is_active());
    }
}
