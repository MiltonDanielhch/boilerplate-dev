// Ubicación: `apps/mobile/src-tauri/src/state.rs`
// 
// Descripción: Estado global de la aplicación Tauri Mobile.
//              Mantiene la conexión a la base de datos local (SQLite)
//              y las instancias de repositorios/servicios.
// 
// ADRs relacionados: 0030 (Multiplataforma Tridente), 0004 (SQLite)

use database::repositories::{CachedUserRepository, SqliteSessionRepository, SqliteUserRepository, SqliteAuditRepository};
use auth::PasetoService;
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{info, error};
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

pub struct AppState {
    pub pool: SqlitePool,
    pub user_repo: Arc<CachedUserRepository>,
    pub session_repo: Arc<SqliteSessionRepository>,
    pub audit_repo: Arc<SqliteAuditRepository>,
    pub paseto: Arc<PasetoService>,
}

impl AppState {
    /// Inicializa el estado global conectándose a la base de datos local y
    /// ejecutando las migraciones.
    pub async fn new(app: &AppHandle) -> anyhow::Result<Self> {
        let app_data_dir = app.path().app_data_dir()
            .expect("No se pudo obtener el directorio de datos de la app");
            
        // Asegurarse de que el directorio de datos existe
        if !app_data_dir.exists() {
            std::fs::create_dir_all(&app_data_dir)?;
        }

        let db_path = app_data_dir.join("boilerplate-mobile.db");
        let db_url = format!("sqlite:{}", db_path.display());
        
        info!("Conectando a base de datos local: {}", db_url);
        
        // Crear pool optimizado con pragmas (ADR 0004)
        let pool = database::pool::create_pool(&db_url).await;
        
        // Ejecutar migraciones embebidas (ADR 0004)
        database::pool::run_migrations(&pool).await?;

        // Inicializar repositorios
        let user_repo = SqliteUserRepository::new(pool.clone());
        let cached_user_repo = Arc::new(CachedUserRepository::new(user_repo));
        let session_repo = Arc::new(SqliteSessionRepository::new(Arc::new(pool.clone())));
        let audit_repo = Arc::new(SqliteAuditRepository::new(pool.clone()));
        
        // Inicializar servicio PASETO
        // Usamos tauri-plugin-store para persistir un secreto único por instalación
        let store = app.store("settings.bin")?;
        
        let paseto_secret = match store.get("paseto_secret") {
            Some(secret_val) => {
                secret_val.as_str().unwrap_or("").to_string()
            },
            None => {
                // Generar un secreto aleatorio de 32 bytes (64 hex chars)
                let mut bytes = [0u8; 32];
                use rand::RngCore;
                rand::thread_rng().fill_bytes(&mut bytes);
                let new_secret = hex::encode(bytes);
                
                store.set("paseto_secret", serde_json::json!(new_secret));
                let _ = store.save();
                new_secret
            }
        };

        if paseto_secret.len() != 64 {
             // Fallback a un secreto por defecto si algo falló (no debería pasar)
             error!("Paseto secret inválido en store, usando fallback");
        }
        
        let paseto = Arc::new(PasetoService::new(&paseto_secret));

        info!("AppState inicializado correctamente");

        Ok(Self {
            pool,
            user_repo: cached_user_repo,
            session_repo,
            audit_repo,
            paseto,
        })
    }
}
