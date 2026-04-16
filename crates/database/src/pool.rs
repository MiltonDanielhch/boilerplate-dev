// Ubicación: `crates/database/src/pool.rs`
//
// Descripción: Pool de conexiones SQLite con PRAGMAs optimizados (ADR 0004).
//              Fail-fast si no conecta (ADR 0002).
//
// ADRs relacionados: ADR 0004 (SQLite WAL), ADR 0002 (Fail-fast)

use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions, Pool, Sqlite,
};
use std::time::Duration;
use tracing::info;

/// Ejecuta las migraciones de SQLx.
pub async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    info!("Ejecutando migraciones...");
    sqlx::migrate!("../../data/migrations").run(pool).await?;
    info!("Migraciones completadas");
    Ok(())
}

/// Crea un pool de conexiones SQLite con configuración optimizada.
///
/// # PRAGMAs aplicados (ADR 0004):
/// - `journal_mode = WAL` — Write-Ahead Logging para concurrencia
/// - `synchronous = NORMAL` — Balance rendimiento/durabilidad
/// - `temp_store = MEMORY` — Temporales en RAM
/// - `mmap_size = 30GB` — Memory-mapped I/O
/// - `foreign_keys = ON` — Integridad referencial
/// - `cache_size = -64000` — 64MB de cache (negativo = páginas)
///
/// # Pool:
/// - max_connections = 10
/// - min_connections = 2
/// - acquire_timeout = 5s
/// - log_slow_statements = 100ms (Warn)
///
/// # Errors
/// Panic si no puede conectar (ADR 0002 — fail-fast).
pub async fn create_pool(database_url: &str) -> Pool<Sqlite> {
    info!("Inicializando pool SQLite: {}", database_url);

    let path = database_url.trim_start_matches("sqlite:");
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .foreign_keys(true)
        .pragma("temp_store", "MEMORY")
        .pragma("mmap_size", "30000000000")
        .pragma("cache_size", "-64000")
        .log_slow_statements(tracing::log::LevelFilter::Warn, Duration::from_millis(100));

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(options)
        .await
        .expect("Failed to connect to SQLite database — ADR 0002 fail-fast");

    info!("Pool SQLite inicializado correctamente");
    pool
}

/// Verifica que el pool esté conectado ejecutando `SELECT 1`.
pub async fn health_check(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").fetch_one(pool).await.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_pool_in_memory() {
        let pool = create_pool(":memory:").await;
        let result = health_check(&pool).await;
        assert!(result.is_ok());
    }
}
