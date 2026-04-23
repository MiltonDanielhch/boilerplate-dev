// Ubicación: `apps/api/src/state.rs`
//
// Descripción: Application State — contiene todas las dependencias inyectadas.
//
// ADRs relacionados: ADR 0001 (Hexagonal), ADR 0003 (Axum)

use auth::PasetoService;
use database::repositories::{SqliteLeadRepository, SqliteSessionRepository, SqliteUserRepository};
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
    pub user_repo: SqliteUserRepository,
    pub session_repo: SqliteSessionRepository,
    pub lead_repo: SqliteLeadRepository,
    pub paseto: Arc<PasetoService>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        user_repo: SqliteUserRepository,
        session_repo: SqliteSessionRepository,
        lead_repo: SqliteLeadRepository,
        paseto: Arc<PasetoService>,
    ) -> Self {
        Self {
            config,
            user_repo,
            session_repo,
            lead_repo,
            paseto,
        }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &self.config)
            .field("user_repo", &self.user_repo)
            .field("session_repo", &"<SqliteSessionRepository>")
            .field("lead_repo", &"<SqliteLeadRepository>")
            .field("paseto", &"<PasetoService>")
            .finish()
    }
}
