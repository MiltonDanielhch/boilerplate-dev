// Ubicación: `apps/api/src/setup.rs`
//
// Descripción: Composition Root — construye el grafo de dependencias.
//              Fail-fast: si algo falla, el proceso no arranca (ADR 0002).
//
// ADRs relacionados: ADR 0001, ADR 0002 (Fail-Fast), ADR 0017 (Caché), ADR 0016 (Observabilidad)

use crate::state::{AppConfig, AppState};
use database::repositories::{
    CachedUserRepository, SqliteAuditRepository, SqliteContentRepository, SqliteLeadRepository, SqliteSessionRepository,
    SqliteSettingsRepository, SqliteUserRepository,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{error, info};

pub fn load_config() -> anyhow::Result<AppConfig> {
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let mut path = std::env::current_exe().unwrap_or_default();
        path.pop();
        path.pop();
        path.push("data");
        path.push("boilerplate.db");
        format!("sqlite:{}", path.display())
    });

    let paseto_secret = std::env::var("PASETO_SECRET").expect("PASETO_SECRET must be set");

    let environment =
        std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    Ok(AppConfig {
        database_url,
        paseto_secret,
        environment,
    })
}

pub fn init_telemetry(config: &AppConfig) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if config.environment == "production" {
            tracing_subscriber::EnvFilter::new("info")
        } else {
            tracing_subscriber::EnvFilter::new("debug")
        }
    });

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(true);

    if config.environment == "production" {
        subscriber.json().init();
    } else {
        subscriber.init();
    }

    if let Ok(sentry_dsn) = std::env::var("SENTRY_DSN") {
        let _sentry = sentry::init((
            sentry_dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                environment: Some(config.environment.clone().into()),
                ..Default::default()
            },
        ));
        info!("Sentry initialized for error tracking");
    } else if config.environment == "production" {
        error!("SENTRY_DSN not set in production - error tracking disabled");
    }

    info!(
        environment = %config.environment,
        "Telemetry initialized"
    );
}

pub fn build_state(pool: SqlitePool, config: AppConfig) -> AppState {
    use auth::PasetoService;

    let user_repo = SqliteUserRepository::new(pool.clone());
    let cached_user_repo = CachedUserRepository::new(user_repo);
    let session_repo = SqliteSessionRepository::new(Arc::new(pool.clone()));
    let audit_repo = SqliteAuditRepository::new(pool.clone());
    let lead_repo = SqliteLeadRepository::new(Arc::new(pool.clone()));
    let content_repo = SqliteContentRepository::new(Arc::new(pool.clone()));
    let settings_repo = SqliteSettingsRepository::new(Arc::new(pool));

    let paseto = Arc::new(PasetoService::new(&config.paseto_secret));

    info!("Application state built successfully");

    AppState::new(
        config,
        cached_user_repo,
        session_repo,
        audit_repo,
        lead_repo,
        content_repo,
        settings_repo,
        paseto,
    )
}
