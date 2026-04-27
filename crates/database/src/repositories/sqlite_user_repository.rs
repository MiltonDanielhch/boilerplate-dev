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
        let created_by = match row.created_by {
            Some(uuid_str) => Some(UserId::parse(&uuid_str).map_err(|_| DomainError::Internal(
                format!("created_by UUID corrupto en DB: {}", uuid_str)
            ))?),
            None => None,
        };

        Ok(User {
            id,
            email,
            password_hash,
            name: row.name,
            is_active: row.is_active,
            email_verified_at: row.email_verified_at,
            last_login_at: row.last_login_at,
            created_by,
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
                   email_verified_at, last_login_at, created_by,
                   created_at, updated_at, deleted_at
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
                   email_verified_at, last_login_at, created_by,
                   created_at, updated_at, deleted_at
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
                             email_verified_at, last_login_at, created_by,
                             created_at, updated_at, deleted_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                email = excluded.email,
                password_hash = excluded.password_hash,
                name = excluded.name,
                is_active = excluded.is_active,
                email_verified_at = excluded.email_verified_at,
                last_login_at = excluded.last_login_at,
                created_by = excluded.created_by,
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
        .bind(user.last_login_at)
        .bind(user.created_by.as_ref().map(|id| id.to_string()))
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

    async fn get_permissions(&self, user_id: &UserId) -> Result<Vec<String>, DomainError> {
        // JOIN 4 tablas: users → user_roles → roles → role_permissions → permissions
        // Retorna lista de permisos en formato "resource:action"
        let permissions: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT p.resource || ':' || p.action as permission
            FROM users u
            JOIN user_roles ur ON ur.user_id = u.id
            JOIN roles r ON r.id = ur.role_id AND r.deleted_at IS NULL
            JOIN role_permissions rp ON rp.role_id = r.id
            JOIN permissions p ON p.id = rp.permission_id
            WHERE u.id = ?
              AND u.deleted_at IS NULL
            ORDER BY p.resource, p.action
            "#
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(permissions)
    }

    async fn has_role(&self, user_id: &UserId, role: &str) -> Result<bool, DomainError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) as count
            FROM user_roles ur
            JOIN roles r ON r.id = ur.role_id AND r.deleted_at IS NULL
            WHERE ur.user_id = ?
              AND r.name = ?
            "#
        )
        .bind(user_id.to_string())
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(count > 0)
    }

    async fn assign_role(&self, user_id: &UserId, role_name: &str) -> Result<(), DomainError> {
        // Buscar el ID del rol por nombre
        let role_id: String = sqlx::query_scalar("SELECT id FROM roles WHERE name = ? AND deleted_at IS NULL")
            .bind(role_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
            .ok_or_else(|| DomainError::Internal(format!("Role '{}' not found", role_name)))?;

        sqlx::query("INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES (?, ?)")
            .bind(user_id.to_string())
            .bind(role_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn remove_role(&self, user_id: &UserId, role_name: &str) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            DELETE FROM user_roles 
            WHERE user_id = ? 
              AND role_id IN (SELECT id FROM roles WHERE name = ?)
            "#
        )
        .bind(user_id.to_string())
        .bind(role_name)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list(
        &self, 
        limit: i64, 
        offset: i64,
        search: Option<String>,
        role: Option<String>,
        is_active: Option<bool>
    ) -> Result<Vec<User>, DomainError> {
        let mut query = String::from(
            r#"
            SELECT DISTINCT u.id, u.email, u.password_hash, u.name, u.is_active,
                   u.email_verified_at, u.last_login_at, u.created_by,
                   u.created_at, u.updated_at, u.deleted_at
            FROM users u
            "#
        );

        if role.is_some() {
            query.push_str("JOIN user_roles ur ON ur.user_id = u.id ");
            query.push_str("JOIN roles r ON r.id = ur.role_id ");
        }

        query.push_str("WHERE u.deleted_at IS NULL ");

        if let Some(s) = &search {
            if !s.is_empty() {
                query.push_str("AND (u.email LIKE ? OR u.name LIKE ?) ");
            }
        }

        if let Some(r) = &role {
            if !r.is_empty() {
                query.push_str("AND r.name = ? ");
            }
        }

        if let Some(active) = is_active {
            if active {
                query.push_str("AND u.is_active = TRUE ");
            } else {
                query.push_str("AND u.is_active = FALSE ");
            }
        }

        query.push_str("ORDER BY u.created_at DESC LIMIT ? OFFSET ?");

        let mut sql_query = sqlx::query_as::<_, UserRow>(&query);

        if let Some(s) = search {
            if !s.is_empty() {
                let pattern = format!("%{}%", s);
                sql_query = sql_query.bind(pattern.clone()).bind(pattern);
            }
        }

        if let Some(r) = role {
            if !r.is_empty() {
                sql_query = sql_query.bind(r);
            }
        }

        sql_query = sql_query.bind(limit).bind(offset);

        let rows = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| self.row_to_entity(r))
            .collect()
    }

    async fn get_counts_by_date(&self, days: i64) -> Result<Vec<(String, i64)>, DomainError> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            WITH RECURSIVE dates(date) AS (
                SELECT DATE('now', '-' || (? - 1) || ' days')
                UNION ALL
                SELECT DATE(date, '+1 day') FROM dates WHERE date < DATE('now')
            )
            SELECT 
                d.date,
                COUNT(u.id) as count
            FROM dates d
            LEFT JOIN users u ON DATE(u.created_at) = d.date AND u.deleted_at IS NULL
            GROUP BY d.date
            ORDER BY d.date ASC
            "#
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows)
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
