// Ubicación: `crates/database/src/models/user_row.rs`
//
// Descripción: UserRow — mapeo exacto de columnas DB para SQLx.
//              Separado de la entidad del dominio.
//
// ADRs relacionados: ADR 0004, ADR 0006 (Soft Delete)

use sqlx::FromRow;
use time::OffsetDateTime;

/// Fila de la tabla `users` — mapeo 1:1 con columnas.
#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub email_verified_at: Option<OffsetDateTime>,
    pub last_login_at: Option<OffsetDateTime>,
    pub created_by: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>, // Soft Delete (ADR 0006)
}

impl UserRow {
    /// Verifica si el usuario está activo (no soft-deleted).
    pub fn is_active(&self) -> bool {
        self.is_active && self.deleted_at.is_none()
    }
}
