// Ubicación: `apps/api/src/state.rs`
//
// Descripción: Application State — contiene todas las dependencias inyectadas.
//
// ADRs relacionados: ADR 0001 (Hexagonal), ADR 0003 (Axum), ADR 0017 (Cache)

use auth::PasetoService;
use database::repositories::{
    CachedUserRepository, SqliteAuditRepository, SqliteContentRepository, SqliteLeadRepository, SqliteSessionRepository,
    SqliteSettingsRepository,
};
use std::sync::Arc;

/// Configuración de la aplicación.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub paseto_secret: String,
    pub environment: String,
}

/// Estado compartido de la aplicación (inyección de dependencias).
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub user_repo: CachedUserRepository,
    pub session_repo: SqliteSessionRepository,
    pub audit_repo: SqliteAuditRepository,
    pub lead_repo: SqliteLeadRepository,
    pub content_repo: SqliteContentRepository,
    pub settings_repo: SqliteSettingsRepository,
    pub paseto: Arc<PasetoService>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        user_repo: CachedUserRepository,
        session_repo: SqliteSessionRepository,
        audit_repo: SqliteAuditRepository,
        lead_repo: SqliteLeadRepository,
        content_repo: SqliteContentRepository,
        settings_repo: SqliteSettingsRepository,
        paseto: Arc<PasetoService>,
    ) -> Self {
        Self {
            config,
            user_repo,
            session_repo,
            audit_repo,
            lead_repo,
            content_repo,
            settings_repo,
            paseto,
        }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &self.config)
            .field("user_repo", &"<CachedUserRepository>")
            .field("session_repo", &"<SqliteSessionRepository>")
            .field("audit_repo", &"<SqliteAuditRepository>")
            .field("lead_repo", &"<SqliteLeadRepository>")
            .field("content_repo", &"<SqliteContentRepository>")
            .field("settings_repo", &"<SqliteSettingsRepository>")
            .field("paseto", &"<PasetoService>")
            .finish()
    }
}
