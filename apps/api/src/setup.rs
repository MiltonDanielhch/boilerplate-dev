// Ubicación: `apps/api/src/setup.rs`
//
// Descripción: Composition Root — construye el grafo de dependencias.
//              Fail-fast: si algo falla, el proceso no arranca (ADR 0002).
//
// ADRs relacionados: ADR 0001, ADR 0002 (Fail-Fast), ADR 0017 (Caché)

use crate::state::{AppConfig, AppState};
use database::repositories::{SqliteSessionRepository, SqliteUserRepository};
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::info;

/// Carga configuración desde variables de entorno (fail-fast).
pub fn load_config() -> anyhow::Result<AppConfig> {
    // Cargar .env si existe (silently ignore error)
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let mut path = std::env::current_exe().unwrap_or_default();
        path.pop(); // bin/debug o bin/release
        path.pop();
        path.push("data");
        path.push("boilerplate.db");
        format!("sqlite:{}", path.display())
    });

    let paseto_secret = std::env::var("PASETO_SECRET").expect("PASETO_SECRET must be set"); // Fail-fast

    let environment =
        std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    Ok(AppConfig {
        database_url,
        paseto_secret,
        environment,
    })
}

/// Inicializa telemetry (tracing).
pub fn init_telemetry(config: &AppConfig) {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .init();

    info!(
        environment = %config.environment,
        "Telemetry initialized"
    );
}

/// Construye el estado de la aplicación (composition root).
pub fn build_state(pool: SqlitePool, config: AppConfig) -> AppState {
    use auth::PasetoService;

    // Repositorios (sin caché por ahora — se agrega en Bloque III)
    let user_repo = SqliteUserRepository::new(pool.clone());
    let session_repo = SqliteSessionRepository::new(Arc::new(pool));

    // TODO: Agregar otros repositorios cuando se implementen
    // let audit_repo = SqliteAuditRepository::new(pool.clone());
    // let token_repo = SqliteTokenRepository::new(pool.clone());
    // let lead_repo = SqliteLeadRepository::new(pool.clone());

    // Servicio PASETO v4 para tokens de acceso (Arc para compartir entre threads)
    let paseto = Arc::new(PasetoService::new(&config.paseto_secret));

    info!("Application state built successfully");

    AppState::new(config, user_repo, session_repo, paseto)
}
