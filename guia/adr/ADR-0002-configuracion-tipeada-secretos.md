# ADR 0002 — Configuración Tipeada: Fail-Fast y Secretos

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0008 (Seguridad — PASETO_SECRET), ADR 0014 (Kamal secrets) |

---

## Contexto

Los problemas más comunes de configuración en producción son:

- **Variables inexistentes** — el proceso truena en mitad de ejecución porque falta una variable
- **Filtramiento de secretos** — subir accidentalmente archivos `.env` a git
- **Falta de tipado** — tratar un puerto (número) como string y obtener errores de conversión oscuros

Necesitamos que la configuración sea validada al arrancar — si falta algo, el proceso no inicia.

---

## Decisión

Usar la crate **`config`** en Rust con un struct centralizado y validación **fail-fast** al arranque.

### El struct de configuración — fuente única de verdad

```rust
// crates/infrastructure/src/config/app_config.rs
use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    // Servidor
    pub server_port:  u16,
    pub environment:  AppEnvironment,
    pub rust_log:     String,

    // Base de datos (ADR 0004)
    pub database_url: String,

    // Autenticación (ADR 0008) — validado como 32 bytes exactos
    pub paseto_secret: String,

    // Email (ADR 0019)
    pub resend_api_key: String,
    pub mail_from:      String,

    // Almacenamiento Tigris/S3 (ADR 0020)
    pub aws_endpoint_url_s3:   String,
    pub aws_access_key_id:     String,
    pub aws_secret_access_key: String,
    pub storage_bucket:        String,

    // Observabilidad (ADR 0016) — opcionales
    pub sentry_dsn:    Option<String>,
    pub otlp_endpoint: Option<String>,

    // Healthchecks.io (ADR 0015) — opcionales
    pub hc_litestream_uuid: Option<String>,
    pub hc_deploy_uuid:     Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppEnvironment {
    Development,
    Staging,
    Production,
}

impl AppConfig {
    /// Carga la configuración desde variables de entorno.
    /// Falla inmediatamente si falta cualquier campo requerido.
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(
                Environment::default()
                    .try_parsing(true), // Parsea tipos automáticamente (u16, bool, etc.)
            )
            .build()?
            .try_deserialize()
    }
}
```

### Validación al arranque — fail-fast intencional

```rust
// apps/api/src/main.rs
#[tokio::main]
async fn main() {
    dotenvy::from_filename(".env.local").ok();

    // Si falta cualquier variable o tiene tipo incorrecto — el proceso termina aquí
    // El error muestra exactamente qué variable falta y por qué
    let config = AppConfig::load()
        .expect("❌ Configuración inválida — revisa las variables de entorno");

    // Validaciones de dominio que config-rs no puede verificar
    assert!(
        config.paseto_secret.len() == 32,
        "❌ PASETO_SECRET debe tener exactamente 32 bytes (usa: openssl rand -hex 16)"
    );

    if config.environment == AppEnvironment::Production {
        assert!(
            config.sentry_dsn.is_some(),
            "❌ SENTRY_DSN es requerido en producción"
        );
    }

    // El servidor arranca solo si la configuración es 100% válida
    tracing::info!(
        environment = ?config.environment,
        port        = config.server_port,
        "configuración válida — iniciando servidor"
    );
}
```

### Jerarquía de fuentes de configuración

```
1. Variables de entorno del sistema   ← Producción (via Kamal secrets)
2. .env.local                         ← Desarrollo local (en .gitignore)
3. .env.example                       ← Plantilla (en git, sin valores reales)
```

### `.env.example` — el contrato público de configuración

```bash
# .env.example — todos los campos requeridos con valores de ejemplo
# Copiar a .env.local y rellenar con valores reales

# Servidor
SERVER_PORT=8080
ENVIRONMENT=development
RUST_LOG=debug,sqlx=warn,tower_http=debug

# Base de datos
DATABASE_URL=sqlite:./data/boilerplate.db

# Autenticación PASETO (exactamente 32 bytes)
# Generar con: openssl rand -hex 16
PASETO_SECRET=cambiar_por_32_bytes_aleatorios__

# Email (ADR 0019)
RESEND_API_KEY=re_xxxxxxxxxxxxxxxxxxxx
MAIL_FROM=Boilerplate <noreply@tudominio.com>

# Almacenamiento Tigris/S3 (ADR 0020)
AWS_ENDPOINT_URL_S3=https://fly.storage.tigris.dev
AWS_ACCESS_KEY_ID=tid_xxxxxxxxxxxxxxxxxxxx
AWS_SECRET_ACCESS_KEY=tsec_xxxxxxxxxxxxxxxxxxxx
STORAGE_BUCKET=boilerplate-development-assets

# Observabilidad — opcionales en desarrollo (ADR 0016)
# SENTRY_DSN=https://xxx@sentry.io/xxx
# OTLP_ENDPOINT=https://otlp.axiom.co/v1/traces

# Healthchecks.io — opcionales (ADR 0015)
# HC_LITESTREAM_UUID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
# HC_DEPLOY_UUID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

---

## Comparativa de estrategias

| Característica | `std::env::var` disperso | Config tipeada (este ADR) |
|----------------|--------------------------|--------------------------|
| **Detección de errores** | Solo al usar la variable — en runtime | Al arrancar — antes de aceptar tráfico |
| **Tipado** | Todo es `String` | Tipos nativos (`u16`, `bool`, enums) |
| **Documentación** | Implícita — hay que leer el código | Explícita — el struct `AppConfig` |
| **Refactoring** | Búsqueda manual de strings | El compilador marca todos los usos |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para elevar la seguridad y la robustez de la gestión de configuración:

| Herramienta | Propósito en la Configuración |
| :--- | :--- |
| **`shadow-rs`** | **Metadatos de Build:** Inyecta versión de git, tiempo de compilación y commit directamente en el binario. |
| **`secrecy`** | **Protección de Secretos:** Envuelve strings sensibles para que nunca se impriman accidentalmente en logs o debuggers. |
| **`nutype`** | **Tipos auto-validados:** Permite que campos como `SERVER_PORT` o `PASETO_SECRET` tengan reglas (ej. rango, longitud) integradas en el tipo. |
| **`figment`** | **Proveedor de Configuración:** Una alternativa moderna a `config-rs` con mejor manejo de errores y validación de esquemas. |
| **`dotenvy`** | **Carga de .env:** El sucesor mantenido de `dotenv`, esencial para el flujo de desarrollo local. |

---

## Consecuencias

### ✅ Positivas

- El struct `AppConfig` es la documentación de todo lo que el sistema necesita para arrancar
- Fail-fast: error claro al arranque, no a mitad de una request de usuario
- Inmutable después de cargar — la configuración no cambia durante la vida del proceso
- `.env.example` en git es el contrato — cualquier developer sabe exactamente qué configurar

### ⚠️ Negativas / Trade-offs

- Cada nueva variable requiere actualizar el struct y recompilar
  → Es una feature, no un bug — el compilador garantiza que nadie olvida actualizar `AppConfig`
  → El PR que agrega una variable nueva DEBE incluir el cambio en el struct
- Sin separador `__` para variables anidadas — se usan nombres FLAT
  → `DATABASE_URL` en lugar de `DATABASE__URL` — más simple y predecible
  → El `.env.example` sirve como referencia exacta del formato correcto

### Decisiones derivadas

- `.env` y `.env.local` están en `.gitignore` — nunca en el repositorio
- `.env.example` está en el repositorio — siempre actualizado antes de cada PR
- `just setup` copia `.env.example` a `.env.local` si no existe (ADR 0012)
- Los secrets en producción se inyectan vía `kamal env set` — nunca en archivos del servidor
- `PASETO_SECRET` se genera con `openssl rand -hex 16` — documentado en `.env.example`
