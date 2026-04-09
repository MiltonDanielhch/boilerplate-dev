// Ubicación: `crates/infrastructure/src/config.rs`
//
// Descripción: Configuración de la aplicación.
//
// ADRs relacionados: ADR 0002

pub struct AppConfig;

impl AppConfig {
    pub fn from_env() -> Self {
        Self
    }
}
