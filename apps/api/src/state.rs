// Ubicación: `apps/api/src/state.rs`
//
// Descripción: Application State — contiene todas las dependencias inyectadas.
//
// ADRs relacionados: ADR 0001 (Hexagonal), ADR 0003 (Axum)

use auth::PasetoService;
use database::repositories::SqliteUserRepository;

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
    pub paseto: PasetoService,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        user_repo: SqliteUserRepository,
        paseto: PasetoService,
    ) -> Self {
        Self {
            config,
            user_repo,
            paseto,
        }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &self.config)
            .field("user_repo", &self.user_repo)
            .field("paseto", &"<PasetoService>")
            .finish()
    }
}
