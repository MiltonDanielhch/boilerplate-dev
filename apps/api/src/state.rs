// Ubicación: `apps/api/src/state.rs`
//
// Descripción: Application State — contiene todas las dependencias inyectadas.
//
// ADRs relacionados: ADR 0001 (Hexagonal), ADR 0003 (Axum)

use database::repositories::SqliteUserRepository;

/// Configuración de la aplicación.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub paseto_secret: String,
    pub environment: String,
}

/// Estado compartido de la aplicación (inyección de dependencias).
#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub user_repo: SqliteUserRepository,
}

impl AppState {
    pub fn new(config: AppConfig, user_repo: SqliteUserRepository) -> Self {
        Self { config, user_repo }
    }
}
