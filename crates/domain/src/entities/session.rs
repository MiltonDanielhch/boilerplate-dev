// Ubicación: `crates/domain/src/entities/session.rs`
//
// Descripción: Entidad Session para tracking de sesiones de usuario.
//              Usada con PASETO tokens (ADR 0008).
//
// ADRs relacionados: ADR 0001, ADR 0008

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Entidad Session del dominio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,           // UUID v7 (session ID)
    pub user_id: String,      // Referencia al usuario
    pub refresh_token_hash: String, // Hash del refresh token (PASETO)
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_revoked: bool,
    pub expires_at: OffsetDateTime,
    pub last_activity_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl Session {
    /// Crea una nueva sesión.
    pub fn new(
        user_id: String,
        refresh_token_hash: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        expires_at: OffsetDateTime,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::now_v7().to_string(),
            user_id,
            refresh_token_hash,
            ip_address,
            user_agent,
            is_revoked: false,
            expires_at,
            last_activity_at: now,
            created_at: now,
        }
    }

    /// Verifica si la sesión es válida (no revocada, no expirada).
    pub fn is_valid(&self) -> bool {
        !self.is_revoked && self.expires_at > OffsetDateTime::now_utc()
    }

    /// Revoca la sesión (logout).
    pub fn revoke(&mut self) {
        self.is_revoked = true;
    }

    /// Verifica si está expirada.
    pub fn is_expired(&self) -> bool {
        self.expires_at <= OffsetDateTime::now_utc()
    }

    /// Actualiza timestamp de última actividad.
    pub fn touch(&mut self) {
        self.last_activity_at = OffsetDateTime::now_utc();
    }
}
