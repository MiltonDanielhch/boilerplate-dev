// Ubicación: `crates/database/src/repositories/sqlite_settings_repository.rs`

use domain::entities::system_setting::SystemSetting;
use domain::errors::DomainError;
use domain::ports::SettingsRepository;
use sqlx::{SqlitePool, FromRow};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SqliteSettingsRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteSettingsRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct SettingRow {
    key: String,
    value: String,
    description: Option<String>,
}

impl From<SettingRow> for SystemSetting {
    fn from(row: SettingRow) -> Self {
        Self {
            key: row.key,
            value: row.value,
            description: row.description,
        }
    }
}

impl SettingsRepository for SqliteSettingsRepository {
    async fn get_setting(&self, key: &str) -> Result<Option<SystemSetting>, DomainError> {
        let row = sqlx::query_as::<_, SettingRow>(
            "SELECT * FROM system_settings WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn save_setting(&self, setting: &SystemSetting) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO system_settings (key, value, description)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                description = excluded.description
            "#
        )
        .bind(&setting.key)
        .bind(&setting.value)
        .bind(&setting.description)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_settings(&self) -> Result<Vec<SystemSetting>, DomainError> {
        let rows = sqlx::query_as::<_, SettingRow>(
            "SELECT * FROM system_settings ORDER BY key ASC"
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
