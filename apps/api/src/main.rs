// Ubicación: `apps/api/src/main.rs`
//
// Descripción: Punto de entrada de la API HTTP.
//              Arranca el servidor Axum con graceful shutdown.
//
// ADRs relacionados: ADR 0003 (Axum), ADR 0001 (Hexagonal), ADR 0002 (Fail-Fast)

use api::{
    router::create_router,
    setup::{build_state, load_config, init_telemetry},
};
use std::net::SocketAddr;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load config (fail-fast — ADR 0002)
    let config = load_config()?;
    
    // 2. Init telemetry
    init_telemetry(&config);
    
    // 3. Create pool
    let pool: sqlx::SqlitePool = database::pool::create_pool(&config.database_url).await;
    
    // 4. Verificar conexión a DB (migraciones se ejecutan con `just migrate`)
    // Nota: sqlx::migrate! macro tiene issues con rutas relativas en Windows
    // Ejecutar: just migrate (antes de iniciar el servidor)
    info!("Database pool created successfully");
    
    // 5. Build application state (composition root)
    let state = build_state(pool, config);
    
    // 6. Build router with middleware
    let app = create_router(state);
    
    // 7. Start server with graceful shutdown
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server starting on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    info!("Server shutdown complete");
    Ok(())
}

/// Graceful shutdown handler (SIGTERM + SIGINT)
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => warn!("Received Ctrl+C, shutting down..."),
        _ = terminate => warn!("Received SIGTERM, shutting down..."),
    }
}
