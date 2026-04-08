# Guía de Inicio — boilerplate

> De cero a un módulo completo funcionando en producción.
> Paso a paso, con código real, sin saltar nada.

---

## Índice

- [Parte 1 — Arrancar el proyecto desde cero](#parte-1--arrancar-el-proyecto-desde-cero)
- [Parte 2 — Tu primer módulo completo](#parte-2--tu-primer-módulo-completo)
- [Parte 3 — Conectar el módulo a la API](#parte-3--conectar-el-módulo-a-la-api)
- [Parte 4 — Tests por capa](#parte-4--tests-por-capa)
- [Parte 5 — El flujo completo de una petición](#parte-5--el-flujo-completo-de-una-petición)

---

# Parte 1 — Arrancar el proyecto desde cero

## Paso 1 — Herramientas que necesitas

```bash
# 1. Rust (si no lo tienes)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Verificar
rustc --version   # 1.82.0 (Edition 2024) o superior
cargo --version

# 3. Herramientas del proyecto (Vía mise - Recomendado 2026)
# El proyecto incluye un .mise.toml que gestiona estas versiones automáticamente
curl https://mise.jdx.dev/install.sh | sh
mise install

# O instalación manual si no usas mise:
cargo install cargo-watch      # hot reload
cargo install cargo-nextest    # test runner 3-5x más rápido
cargo install cargo-deny       # auditoría de licencias
cargo install cargo-audit      # vulnerabilidades conocidas
cargo install sqlx-cli --features sqlite
cargo install just
cargo install lefthook
cargo install bacon            # background code checker (opcional)

# 4. Node.js 20+ y pnpm
npm install -g pnpm

# Verificar todo
cargo watch --version
cargo nextest --version
sqlx --version
just --version
pnpm --version
lefthook --version
```

---

## Paso 2 — Crear el repositorio y el workspace

```bash
mkdir boilerplate
cd boilerplate
git init
```

Crear el `.gitignore`:

```gitignore
# Rust
/target
**/*.rs.bk

# Cargo.lock se INCLUYE en git — este es un binario, no una librería
# (eliminar la línea de Cargo.lock si la tienes)

# Variables de entorno — NUNCA en git
.env.local
.env

# Datos locales
/data
*.db *.db-wal *.db-shm

# Node
node_modules
.pnpm-store
apps/web/.astro
apps/web/dist
apps/mailer/dist

# IDE
.idea .vscode *.swp
```

Crear el `Cargo.toml` del workspace:

```toml
[workspace]
resolver = "2"
members  = [
    "apps/api",
    "apps/cli",
    "crates/domain",
    "crates/application",
    "crates/infrastructure",
    "crates/database",
    "crates/auth",
    "crates/mailer",
    "crates/storage",
    "crates/events",
]

[workspace.dependencies]
# Runtime
tokio      = { version = "1",   features = ["full"] }
serde      = { version = "1",   features = ["derive"] }
serde_json = "1"
uuid       = { version = "1",   features = ["v4", "v7", "serde"] }
time       = { version = "0.3", features = ["serde", "macros", "formatting", "parsing"] }

# Errores
anyhow    = "1"
thiserror = "2"

# DB
sqlx = { version = "0.8", features = [
    "sqlite", "runtime-tokio", "macros", "uuid", "time"
] }

# Web
axum           = { version = "0.8", features = ["macros"] }
axum-extra     = { version = "0.9", features = ["cookie", "typed-header"] }
tower          = "0.5"
tower-http     = { version = "0.6", features = [
    "cors", "compression-gzip", "compression-br",
    "trace", "timeout", "request-id"
] }
tower-governor = "0.4"

# Auth — PASETO v4, sin JWT
argon2   = "0.5"
pasetors = { version = "0.7", features = ["v4"] }

# Cache y jobs
moka       = { version = "0.12", features = ["future"] }
apalis     = { version = "0.6",  features = ["sqlite", "tracing", "retry"] }
apalis-sql = { version = "0.6",  features = ["sqlite"] }

# Observabilidad
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
sentry             = "0.34"

# Docs API
utoipa        = { version = "4", features = ["axum_extras", "uuid"] }
utoipa-scalar = { version = "0.1", features = ["axum"] }

# Config
config = "0.14"

# Email
resend-rs = "0.5"

# Storage
aws-config = "1.1"
aws-sdk-s3 = "1.1"

# Eventos (Fase 2 — activar cuando se necesite)
async-nats = "0.36"

[workspace.dev-dependencies]
tokio    = { version = "1", features = ["test", "macros"] }
mockall  = "0.13"
reqwest  = { version = "0.12", features = ["json"] }
httpmock = "0.7"
```

---

## Paso 3 — Crear la estructura de carpetas

```bash
# Apps
mkdir -p apps/api/src
mkdir -p apps/cli/src

# Crates — cada uno con su propio Cargo.toml
mkdir -p crates/domain/src/{entities,value_objects,ports,errors}
mkdir -p crates/application/src/use_cases/{auth,users,roles}
mkdir -p crates/infrastructure/src/{http/{handlers,middleware},config,docs}
mkdir -p crates/database/src/{repositories,models}
mkdir -p crates/auth/src
mkdir -p crates/mailer/src
mkdir -p crates/storage/src
mkdir -p crates/events/src

# Datos y migraciones
mkdir -p data/migrations
mkdir -p data/seeds

# Infraestructura de deploy
mkdir -p infra/docker
mkdir -p infra/caddy
mkdir -p infra/litestream
mkdir -p infra/kamal

# Protobuf (Fase 2)
mkdir -p proto/user/v1
mkdir -p proto/common/v1

# Docs y ADRs
mkdir -p docs/adr/future

echo "✅ Estructura creada"
```

---

## Paso 4 — Crear los crates con sus `Cargo.toml`

El `Cargo.toml` de cada crate es lo que hace cumplir las fronteras de arquitectura.
Si `crates/domain/Cargo.toml` no declara `sqlx`, es imposible importarlo desde domain.

```bash
# ── crates/domain ── solo thiserror, uuid, time, serde ─────────────────────
cat > crates/domain/Cargo.toml << 'EOF'
[package]
name    = "domain"
version = "0.1.0"
edition = "2024"

[dependencies]
thiserror = { workspace = true }
uuid      = { workspace = true }
time      = { workspace = true }
serde     = { workspace = true }
# NADA MÁS — este límite es la garantía de la arquitectura hexagonal
EOF

cat > crates/domain/src/lib.rs << 'EOF'
pub mod entities;
pub mod value_objects;
pub mod ports;
pub mod errors;
EOF

# ── crates/application ── solo domain ──────────────────────────────────────
cat > crates/application/Cargo.toml << 'EOF'
[package]
name    = "application"
version = "0.1.0"
edition = "2024"

[dependencies]
domain  = { path = "../domain" }
tracing = { workspace = true }
uuid    = { workspace = true }

[dev-dependencies]
mockall = { workspace = true }
tokio   = { workspace = true }
EOF

cat > crates/application/src/lib.rs << 'EOF'
pub mod use_cases;
EOF

# ── crates/database ── domain + sqlx + moka ────────────────────────────────
cat > crates/database/Cargo.toml << 'EOF'
[package]
name    = "database"
version = "0.1.0"
edition = "2024"

[dependencies]
domain  = { path = "../domain" }
sqlx    = { workspace = true }
moka    = { workspace = true }
uuid    = { workspace = true }
time    = { workspace = true }
tracing = { workspace = true }
EOF

cat > crates/database/src/lib.rs << 'EOF'
pub mod pool;
pub mod repositories;
pub mod models;

pub use pool::create_pool;
EOF

# ── crates/auth ── domain + argon2 + pasetors ──────────────────────────────
cat > crates/auth/Cargo.toml << 'EOF'
[package]
name    = "auth"
version = "0.1.0"
edition = "2024"

[dependencies]
domain   = { path = "../domain" }
argon2   = { workspace = true }
pasetors = { workspace = true }
uuid     = { workspace = true }
time     = { workspace = true }
thiserror = { workspace = true }
EOF

cat > crates/auth/src/lib.rs << 'EOF'
pub mod password;
pub mod paseto;
pub mod token;
EOF

# ── crates/mailer ── domain + resend-rs ────────────────────────────────────
cat > crates/mailer/Cargo.toml << 'EOF'
[package]
name    = "mailer"
version = "0.1.0"
edition = "2024"

[dependencies]
domain    = { path = "../domain" }
resend-rs = { workspace = true }
tracing   = { workspace = true }
EOF

cat > crates/mailer/src/lib.rs << 'EOF'
pub mod log_mailer;
pub mod resend_mailer;
EOF

# ── crates/storage ── domain + aws ─────────────────────────────────────────
cat > crates/storage/Cargo.toml << 'EOF'
[package]
name    = "storage"
version = "0.1.0"
edition = "2024"

[dependencies]
domain     = { path = "../domain" }
aws-config = { workspace = true }
aws-sdk-s3 = { workspace = true }
tracing    = { workspace = true }
EOF

cat > crates/storage/src/lib.rs << 'EOF'
pub mod tigris_repository;
EOF

# ── crates/events ── domain + async-nats (Fase 2) ──────────────────────────
cat > crates/events/Cargo.toml << 'EOF'
[package]
name    = "events"
version = "0.1.0"
edition = "2024"

[dependencies]
domain     = { path = "../domain" }
async-nats = { workspace = true }
serde      = { workspace = true }
serde_json = { workspace = true }
tracing    = { workspace = true }
EOF

cat > crates/events/src/lib.rs << 'EOF'
pub mod publisher;
EOF

# ── crates/infrastructure ── ensambla todo ─────────────────────────────────
cat > crates/infrastructure/Cargo.toml << 'EOF'
[package]
name    = "infrastructure"
version = "0.1.0"
edition = "2024"

[dependencies]
domain         = { path = "../domain" }
application    = { path = "../application" }
database       = { path = "../database" }
auth           = { path = "../auth" }
mailer         = { path = "../mailer" }
storage        = { path = "../storage" }
axum           = { workspace = true }
axum-extra     = { workspace = true }
tower          = { workspace = true }
tower-http     = { workspace = true }
tower-governor = { workspace = true }
tokio          = { workspace = true }
serde          = { workspace = true }
serde_json     = { workspace = true }
uuid           = { workspace = true }
time           = { workspace = true }
tracing        = { workspace = true }
config         = { workspace = true }
utoipa         = { workspace = true }
utoipa-scalar  = { workspace = true }
EOF

cat > crates/infrastructure/src/lib.rs << 'EOF'
pub mod http;
pub mod config;
pub mod docs;
EOF

# ── apps/api ── punto de entrada del servidor ──────────────────────────────
cat > apps/api/Cargo.toml << 'EOF'
[package]
name    = "api"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "api"
path = "src/main.rs"

[dependencies]
infrastructure = { path = "../../crates/infrastructure" }
database       = { path = "../../crates/database" }
auth           = { path = "../../crates/auth" }
mailer         = { path = "../../crates/mailer" }
storage        = { path = "../../crates/storage" }
tokio          = { workspace = true }
tracing        = { workspace = true }
tracing-subscriber = { workspace = true }
sentry         = { workspace = true }
anyhow         = { workspace = true }
EOF

echo 'fn main() { println!("boilerplate"); }' > apps/api/src/main.rs

# ── apps/cli ── Sintonía CLI ───────────────────────────────────────────────
cat > apps/cli/Cargo.toml << 'EOF'
[package]
name    = "cli"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "sintonia"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
EOF

echo 'fn main() { println!("sintonia cli"); }' > apps/cli/src/main.rs

echo "✅ Crates creados"
```

Verificar que el workspace compila limpio:

```bash
cargo check --workspace
# Sin errores — si hay alguno, revisar las rutas en path = "../..."
```

---

## Paso 5 — Las 6 migraciones del sistema (ADR 0025)

```bash
export DATABASE_URL="sqlite:./data/boilerplate.db"
sqlx database create
```

Copia los 6 archivos de migraciones en `data/migrations/`. El orden importa:

```bash
# Verificar que están todos
ls data/migrations/
# 20260305135148_create_users_table.sql
# 20260305135149_create_rbac.sql
# 20260305135150_create_tokens.sql
# 20260305135151_create_audit_logs.sql
# 20260305135152_seed_system_data.sql
# 20260305135153_create_sessions.sql

# Ejecutar
sqlx migrate run
```

Output esperado:

```
Applied 20260305135148/migrate create_users_table
Applied 20260305135149/migrate create_rbac
Applied 20260305135150/migrate create_tokens
Applied 20260305135151/migrate create_audit_logs
Applied 20260305135152/migrate seed_system_data
Applied 20260305135153/migrate create_sessions
```

Verificar:

```bash
sqlite3 ./data/boilerplate.db ".tables"
# audit_logs  permissions  role_permissions  roles
# sessions    tokens       user_roles        users
```

> ⚠️ El seed instala `admin@admin.com` con password `12345678`.
> Cambiar antes del primer deploy a producción.

---

## Paso 6 — Variables de entorno (ADR 0019)

```bash
cat > .env.example << 'EOF'
# Servidor
SERVER_PORT=8080
ENVIRONMENT=development
RUST_LOG=debug,sqlx=warn,tower_http=debug

# Base de datos
DATABASE_URL=sqlite:./data/boilerplate.db

# Auth — PASETO v4 (exactamente 32 bytes — generar con: openssl rand -hex 16)
PASETO_SECRET=cambiar_por_32_bytes_aleatorios__

# Email (ADR 0022)
RESEND_API_KEY=re_xxxxxxxxxxxxxxxxxxxx
MAIL_FROM=Boilerplate <noreply@tudominio.com>

# Storage — Tigris/S3 (ADR 0013)
AWS_ENDPOINT_URL_S3=https://fly.storage.tigris.dev
AWS_ACCESS_KEY_ID=tid_xxxxxxxxxxxxxxxxxxxx
AWS_SECRET_ACCESS_KEY=tsec_xxxxxxxxxxxxxxxxxxxx
STORAGE_BUCKET=boilerplate-development-assets

# Observabilidad (ADR 0004) — opcionales en desarrollo
# SENTRY_DSN=https://xxx@sentry.io/xxx
# OTLP_ENDPOINT=https://otlp.axiom.co/v1/traces

# Healthchecks.io (ADR 0015) — opcionales
# HC_LITESTREAM_UUID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
# HC_DEPLOY_UUID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
EOF

cp .env.example .env.local
# Editar .env.local con valores reales
```

---

## Paso 7 — justfile + lefthook

```bash
cat > justfile << 'EOF'
# justfile — boilerplate
# Ver todos los comandos: just --list

setup:
    cargo install cargo-watch cargo-nextest cargo-deny cargo-audit sqlx-cli lefthook
    npm install -g pnpm
    pnpm install
    cp -n .env.example .env.local || true
    lefthook install
    sqlx database create
    just migrate
    @echo "✅ Setup completo — edita .env.local y ejecuta: just dev"

dev:
    cargo watch -x "run --bin api" & pnpm --filter web dev

dev-api:
    cargo watch -x "run --bin api"

build:
    pnpm --filter mailer build
    cargo build --release
    pnpm --filter web build

test:
    cargo nextest run

test-all:
    cargo nextest run --all-targets

test-v:
    cargo nextest run --no-capture

lint:
    cargo clippy --all-targets -- -D warnings

fmt:
    cargo fmt --all

check:
    cargo check --workspace

audit:
    cargo deny check
    cargo audit

migrate:
    sqlx migrate run

migrate-reset:
    sqlx database reset

migrate-new name:
    sqlx migrate add {{name}}

db-status:
    sqlx migrate info

prepare:
    cargo sqlx prepare --workspace

deploy:
    just audit
    just test
    kamal deploy
    curl -fsS ${HC_DEPLOY_UUID:+https://hc-ping.com/$HC_DEPLOY_UUID} || true

redeploy:
    kamal redeploy

rollback:
    kamal rollback

logs:
    kamal logs -f

status:
    kamal details
EOF

cat > lefthook.yml << 'EOF'
pre-commit:
    parallel: true
    commands:
        fmt-rust:
            run:  cargo fmt --all --check
            glob: "*.rs"
        check:
            run:  cargo check --workspace

pre-push:
    commands:
        lint:
            run: cargo clippy --all-targets -- -D warnings
        test:
            run: cargo nextest run
        audit:
            run: cargo deny check
EOF

lefthook install
```

---

## Paso 8 — Commit inicial

```bash
git add .
git commit -m "chore: setup inicial del workspace boilerplate"

echo "✅ Proyecto inicializado"
echo "   Próximo paso: just dev y construir el primer módulo"
```

---

# Parte 2 — Tu primer módulo completo (User)

El módulo `User` cubre el 80% de los patrones que usarás en todos los demás.
Orden obligatorio: **Domain → Database → Application → Infrastructure → Tests**.

---

## Paso 9 — Domain: Value Objects

### `UserId`

```rust
// crates/domain/src/value_objects/user_id.rs
use uuid::Uuid;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }

    pub fn from_str(s: &str) -> Result<Self, crate::errors::DomainError> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| crate::errors::DomainError::InvalidId(s.to_string()))
    }

    pub fn as_str(&self) -> String { self.0.to_string() }
}

impl Default for UserId {
    fn default() -> Self { Self::new() }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

### `Email`

```rust
// crates/domain/src/value_objects/email.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn new(raw: &str) -> Result<Self, crate::errors::DomainError> {
        let normalized = raw.trim().to_lowercase();

        if normalized.is_empty() {
            return Err(crate::errors::DomainError::InvalidEmail(
                "email no puede estar vacío".into()
            ));
        }

        let parts: Vec<&str> = normalized.splitn(2, '@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(crate::errors::DomainError::InvalidEmail(
                format!("'{}' no es un email válido", raw)
            ));
        }

        if !parts[1].contains('.') {
            return Err(crate::errors::DomainError::InvalidEmail(
                format!("dominio '{}' no es válido", parts[1])
            ));
        }

        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

### `PasswordHash`

```rust
// crates/domain/src/value_objects/password_hash.rs

/// El dominio solo necesita saber que existe un hash.
/// Cómo se genera es responsabilidad de crates/auth.
#[derive(Debug, Clone)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn from_hash(hash: String) -> Self { Self(hash) }
    pub fn as_str(&self) -> &str { &self.0 }
}
// No implementamos Display — nunca imprimir un hash accidentalmente
```

```rust
// crates/domain/src/value_objects/mod.rs
pub mod user_id;
pub mod email;
pub mod password_hash;

pub use user_id::UserId;
pub use email::Email;
pub use password_hash::PasswordHash;
```

---

## Paso 10 — Domain: Errors (ADR 0024)

```rust
// crates/domain/src/errors/domain_error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("ID inválido: {0}")]
    InvalidId(String),

    #[error("Email inválido: {0}")]
    InvalidEmail(String),

    #[error("Fecha inválida")]
    InvalidDate,

    #[error("Usuario no encontrado")]
    UserNotFound,

    #[error("el email ya está registrado")]
    EmailAlreadyExists,

    #[error("credenciales inválidas")]
    InvalidCredentials,

    #[error("token expirado o revocado")]
    InvalidToken,

    #[error("no tienes permiso para: {reason}")]
    Forbidden { reason: String },

    // Error de infraestructura — mapeado sin exponer detalles al cliente
    #[error("error de base de datos")]
    Database(String),
}

// Conversión desde sqlx::Error — solo en crates/database, nunca en domain
// (aceptable en MVP, documentado como deuda técnica)
impl From<sqlx::Error> for DomainError {
    fn from(e: sqlx::Error) -> Self {
        Self::Database(e.to_string())
    }
}
```

```rust
// crates/domain/src/errors/app_error.rs
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use super::DomainError;

#[derive(Debug)]
pub enum AppError {
    Domain(DomainError),
    Unauthorized,
    TokenExpired,
    Internal(String),
}

impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self { Self::Domain(e) }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Domain(DomainError::InvalidEmail(m))   => (StatusCode::BAD_REQUEST,   "invalid_email",          m.clone()),
            AppError::Domain(DomainError::EmailAlreadyExists) => (StatusCode::CONFLICT,      "email_already_exists",   self.to_string()),
            AppError::Domain(DomainError::UserNotFound)       => (StatusCode::NOT_FOUND,     "not_found",              self.to_string()),
            AppError::Domain(DomainError::Forbidden { .. })  => (StatusCode::FORBIDDEN,     "forbidden",              self.to_string()),
            AppError::Domain(DomainError::InvalidCredentials) => (StatusCode::UNAUTHORIZED,  "invalid_credentials",    self.to_string()),
            AppError::Unauthorized                            => (StatusCode::UNAUTHORIZED,  "unauthorized",           "no autenticado".into()),
            AppError::TokenExpired                            => (StatusCode::UNAUTHORIZED,  "token_expired",          "token expirado".into()),
            AppError::Domain(DomainError::Database(_))
            | AppError::Internal(_)                           => {
                tracing::error!(error = ?self, "error interno");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Error interno del servidor".into())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Error interno del servidor".into()),
        };

        (status, Json(json!({ "error": code, "message": message }))).into_response()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Domain(e)   => write!(f, "{}", e),
            Self::Unauthorized => write!(f, "no autenticado"),
            Self::TokenExpired => write!(f, "token expirado"),
            Self::Internal(m) => write!(f, "{}", m),
        }
    }
}
```

```rust
// crates/domain/src/errors/mod.rs
pub mod domain_error;
pub mod app_error;
pub use domain_error::DomainError;
pub use app_error::AppError;
```

---

## Paso 11 — Domain: Entity User

```rust
// crates/domain/src/entities/user.rs
use time::OffsetDateTime;
use crate::value_objects::{UserId, Email, PasswordHash};

#[derive(Debug, Clone)]
pub struct User {
    pub id:             UserId,
    pub username:       Option<String>,
    pub email:          Email,
    pub password_hash:  PasswordHash,
    pub email_verified: bool,
    pub created_at:     OffsetDateTime,
    pub updated_at:     OffsetDateTime,
    pub deleted_at:     Option<OffsetDateTime>, // Soft Delete
}

impl User {
    pub fn new(email: Email, password_hash: PasswordHash) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id:             UserId::new(),
            username:       None,
            email,
            password_hash,
            email_verified: false,
            created_at:     now,
            updated_at:     now,
            deleted_at:     None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    pub fn soft_delete(&mut self) {
        self.deleted_at = Some(OffsetDateTime::now_utc());
    }
}
```

```rust
// crates/domain/src/entities/mod.rs
pub mod user;
pub use user::User;
```

---

## Paso 12 — Domain: Ports (ADR 0000, 0025)

```rust
// crates/domain/src/ports/user_repository.rs
use crate::{entities::User, errors::DomainError, value_objects::{UserId, Email}};

pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &UserId)             -> Result<Option<User>, DomainError>;
    async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;
    async fn save(&self, user: &User)                   -> Result<(), DomainError>;
    async fn soft_delete(&self, id: &UserId)            -> Result<(), DomainError>;
    async fn list(&self, limit: i64, offset: i64)       -> Result<Vec<User>, DomainError>;
    async fn count(&self)                               -> Result<i64, DomainError>;
    async fn has_permission(&self, id: &UserId, perm: &str) -> Result<bool, DomainError>;
}
```

```rust
// crates/domain/src/ports/mod.rs
pub mod user_repository;
pub use user_repository::UserRepository;
```

Actualizar `crates/domain/src/lib.rs`:

```rust
pub mod entities;
pub mod value_objects;
pub mod ports;
pub mod errors;

pub use entities::User;
pub use value_objects::{UserId, Email, PasswordHash};
pub use errors::{DomainError, AppError};
pub use ports::UserRepository;
```

```bash
cargo check -p domain
# Sin errores
```

---

## Paso 13 — Database: Pool + Row + Repositorio (ADR 0002)

```rust
// crates/database/src/pool.rs
use sqlx::{sqlite::{SqlitePoolOptions, SqliteConnectOptions}, SqlitePool};
use std::{str::FromStr, time::Duration};
use tracing::log::LevelFilter;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(
            SqliteConnectOptions::from_str(database_url)?
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
                .pragma("temp_store",   "MEMORY")
                .pragma("mmap_size",    "30000000000")
                .pragma("foreign_keys", "ON")
                .pragma("cache_size",   "-64000")
                .log_statements(LevelFilter::Debug)
                .log_slow_statements(LevelFilter::Warn, Duration::from_millis(100)),
        )
        .await
}
```

```rust
// crates/database/src/models/user_row.rs
use sqlx::FromRow;

/// Mapeo directo de columnas SQL.
/// SEPARADO de la entidad User del dominio — son conceptos distintos.
#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id:             String,
    pub username:       Option<String>,
    pub email:          String,
    pub password_hash:  String,
    pub email_verified: bool,
    pub created_at:     String,
    pub updated_at:     String,
    pub deleted_at:     Option<String>,
}
```

```rust
// crates/database/src/repositories/sqlite_user_repository.rs
use std::sync::Arc;
use sqlx::SqlitePool;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use domain::{User, UserId, Email, PasswordHash, DomainError, UserRepository};
use crate::models::UserRow;

pub struct SqliteUserRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteUserRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self { Self { pool } }
}

impl UserRepository for SqliteUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        sqlx::query_as!(UserRow,
            "SELECT id, username, email, password_hash, email_verified,
                    created_at, updated_at, deleted_at
             FROM users WHERE id = ?",
            id.as_str()
        )
        .fetch_optional(&*self.pool).await?
        .map(row_to_user).transpose()
    }

    async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        // El índice parcial idx_users_email_active hace esto ultra-rápido
        sqlx::query_as!(UserRow,
            "SELECT id, username, email, password_hash, email_verified,
                    created_at, updated_at, deleted_at
             FROM users WHERE email = ? AND deleted_at IS NULL",
            email.as_str()
        )
        .fetch_optional(&*self.pool).await?
        .map(row_to_user).transpose()
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        let created = user.created_at.format(&Rfc3339).map_err(|_| DomainError::InvalidDate)?;
        let updated = user.updated_at.format(&Rfc3339).map_err(|_| DomainError::InvalidDate)?;

        sqlx::query!(
            "INSERT INTO users (id, username, email, password_hash, email_verified, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            user.id.as_str(),
            user.username,
            user.email.as_str(),
            user.password_hash.as_str(),
            user.email_verified,
            created,
            updated,
        )
        .execute(&*self.pool).await?;
        Ok(())
    }

    async fn soft_delete(&self, id: &UserId) -> Result<(), DomainError> {
        // UPDATE deleted_at — nunca DELETE real (ADR 0025)
        let now = OffsetDateTime::now_utc().format(&Rfc3339)
            .map_err(|_| DomainError::InvalidDate)?;
        let affected = sqlx::query!(
            "UPDATE users SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL",
            now, id.as_str()
        )
        .execute(&*self.pool).await?.rows_affected();

        if affected == 0 { return Err(DomainError::UserNotFound); }
        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>, DomainError> {
        sqlx::query_as!(UserRow,
            "SELECT id, username, email, password_hash, email_verified,
                    created_at, updated_at, deleted_at
             FROM users WHERE deleted_at IS NULL
             ORDER BY created_at DESC LIMIT ? OFFSET ?",
            limit, offset
        )
        .fetch_all(&*self.pool).await?
        .into_iter().map(row_to_user).collect()
    }

    async fn count(&self) -> Result<i64, DomainError> {
        Ok(sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE deleted_at IS NULL"
        ).fetch_one(&*self.pool).await?)
    }

    async fn has_permission(&self, id: &UserId, perm: &str) -> Result<bool, DomainError> {
        // JOIN de 4 tablas — invisible para el dominio (ADR 0025)
        let count = sqlx::query_scalar!(r#"
            SELECT COUNT(*) as "count: i64"
            FROM users u
            JOIN user_roles ur       ON ur.user_id       = u.id
            JOIN role_permissions rp ON rp.role_id       = ur.role_id
            JOIN permissions p       ON p.id             = rp.permission_id
            WHERE u.id = ? AND p.name = ? AND u.deleted_at IS NULL
        "#, id.as_str(), perm)
        .fetch_one(&*self.pool).await?;
        Ok(count > 0)
    }
}

fn row_to_user(row: UserRow) -> Result<User, DomainError> {
    Ok(User {
        id:             UserId::from_str(&row.id)?,
        username:       row.username,
        email:          Email::new(&row.email)?,
        password_hash:  PasswordHash::from_hash(row.password_hash),
        email_verified: row.email_verified,
        created_at:     parse_dt(&row.created_at)?,
        updated_at:     parse_dt(&row.updated_at)?,
        deleted_at:     row.deleted_at.map(|s| parse_dt(&s)).transpose()?,
    })
}

fn parse_dt(s: &str) -> Result<OffsetDateTime, DomainError> {
    OffsetDateTime::parse(s, &Rfc3339).map_err(|_| DomainError::InvalidDate)
}
```

```bash
cargo check -p database
```

---

## Paso 14 — crates/auth: argon2id + PASETO v4 (ADR 0008)

```rust
// crates/auth/src/password.rs
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use domain::DomainError;

pub fn hash_password(password: &str) -> Result<String, DomainError> {
    let salt   = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Parámetros OWASP 2024
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| DomainError::Database(e.to_string()))?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, DomainError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| DomainError::Database(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
```

```rust
// crates/auth/src/paseto.rs
use pasetors::v4::local::{LocalKey, encode, decode};
use pasetors::claims::Claims;
use time::{Duration, OffsetDateTime};
use domain::DomainError;

pub struct PasetoService {
    key: LocalKey,
}

impl PasetoService {
    /// Panics si secret no tiene exactamente 32 bytes — fail-fast intencional
    pub fn new(secret: &str) -> Self {
        assert!(
            secret.len() == 32,
            "PASETO_SECRET debe tener exactamente 32 bytes, tiene {}",
            secret.len()
        );
        let key = LocalKey::from(secret.as_bytes().try_into().unwrap());
        Self { key }
    }

    pub fn generate(&self, user_id: &str) -> Result<String, DomainError> {
        let mut claims = Claims::new().map_err(|e| DomainError::Database(e.to_string()))?;
        claims.add_additional("sub", user_id)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        claims.expiration(&(OffsetDateTime::now_utc() + Duration::minutes(15)))
            .map_err(|e| DomainError::Database(e.to_string()))?;

        encode(&self.key, &claims, None, Some(b"boilerplate-v1"))
            .map_err(|e| DomainError::Database(e.to_string()))
    }

    pub fn verify(&self, token: &str) -> Result<String, DomainError> {
        let claims = decode(&self.key, token, None, Some(b"boilerplate-v1"))
            .map_err(|_| DomainError::InvalidToken)?;

        claims.get_claim("sub")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or(DomainError::InvalidToken)
    }
}
```

---

## Paso 15 — Application: Casos de uso

```rust
// crates/application/src/use_cases/users/create_user.rs
use std::sync::Arc;
use domain::{User, Email, PasswordHash, DomainError, UserRepository};

pub struct CreateUserInput {
    pub email:    String,
    pub password: String,
}

pub struct CreateUserOutput {
    pub id:    String,
    pub email: String,
}

pub struct CreateUserUseCase<R: UserRepository> {
    pub repo:     Arc<R>,
    pub hash_fn:  Arc<dyn Fn(&str) -> Result<String, DomainError> + Send + Sync>,
}

impl<R: UserRepository> CreateUserUseCase<R> {
    pub async fn execute(&self, input: CreateUserInput) -> Result<CreateUserOutput, DomainError> {
        let email = Email::new(&input.email)?;

        // Verificar unicidad solo entre usuarios activos — índice parcial lo hace rápido
        if self.repo.find_active_by_email(&email).await?.is_some() {
            return Err(DomainError::EmailAlreadyExists);
        }

        let hash          = (self.hash_fn)(&input.password)?;
        let password_hash = PasswordHash::from_hash(hash);
        let user          = User::new(email, password_hash);

        self.repo.save(&user).await?;

        tracing::info!(user_id = %user.id, "usuario creado");

        Ok(CreateUserOutput {
            id:    user.id.as_str(),
            email: user.email.as_str().to_string(),
        })
    }
}
```

```rust
// crates/application/src/use_cases/users/get_user.rs
use std::sync::Arc;
use domain::{DomainError, UserRepository, UserId, User};

pub struct GetUserUseCase<R: UserRepository> {
    pub repo: Arc<R>,
}

impl<R: UserRepository> GetUserUseCase<R> {
    pub async fn execute(&self, id: &str) -> Result<User, DomainError> {
        let user_id = UserId::from_str(id)?;
        self.repo
            .find_by_id(&user_id).await?
            .filter(|u| u.is_active())
            .ok_or(DomainError::UserNotFound)
    }
}
```

```rust
// crates/application/src/use_cases/users/mod.rs
pub mod create_user;
pub mod get_user;

pub use create_user::{CreateUserUseCase, CreateUserInput, CreateUserOutput};
pub use get_user::GetUserUseCase;
```

---

## Paso 16 — Infrastructure: AppState + handlers

```rust
// crates/infrastructure/src/config/app_config.rs
use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server_port:  u16,
    pub environment:  String,
    pub database_url: String,
    pub paseto_secret: String,
    pub resend_api_key: Option<String>,
    pub mail_from:      Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(Environment::default().try_parsing(true))
            .build()?
            .try_deserialize()
    }
}
```

```rust
// crates/infrastructure/src/http/state.rs
use std::sync::Arc;
use domain::UserRepository;
use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub user_repo: Arc<dyn UserRepository>,
    pub config:    Arc<AppConfig>,
}
```

```rust
// crates/infrastructure/src/http/handlers/users.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use application::use_cases::users::*;
use auth::password::hash_password;
use domain::AppError;
use crate::http::state::AppState;

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]     pub page:     i64,
    #[serde(default = "default_per_page")] pub per_page: i64,
}
fn default_page()     -> i64 { 1  }
fn default_per_page() -> i64 { 20 }

#[derive(Serialize)]
pub struct UserResponse { pub id: String, pub email: String }

#[derive(Deserialize)]
pub struct CreateUserRequest { pub email: String, pub password: String }

// GET /api/v1/users
pub async fn list_users(
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = state.user_repo.list(p.per_page, (p.page - 1) * p.per_page).await
        .map_err(AppError::Domain)?;
    Ok(Json(users.into_iter().map(|u| UserResponse {
        id:    u.id.as_str(),
        email: u.email.as_str().to_string(),
    }).collect()))
}

// GET /api/v1/users/:id
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UserResponse>, AppError> {
    let user = GetUserUseCase { repo: state.user_repo }
        .execute(&id).await
        .map_err(AppError::Domain)?;
    Ok(Json(UserResponse { id: user.id.as_str(), email: user.email.as_str().to_string() }))
}

// POST /api/v1/users
pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    let out = CreateUserUseCase {
        repo:    state.user_repo,
        hash_fn: Arc::new(|p| hash_password(p)),
    }
    .execute(CreateUserInput { email: body.email, password: body.password })
    .await
    .map_err(AppError::Domain)?;

    Ok((StatusCode::CREATED, Json(UserResponse { id: out.id, email: out.email })))
}
```

```rust
// crates/infrastructure/src/http/router.rs
use axum::{Router, routing::{get, post}};
use crate::http::{handlers::users, state::AppState};

pub fn api_router() -> Router<AppState> {
    Router::new().nest("/api/v1",
        Router::new()
            .route("/users",     get(users::list_users).post(users::create_user))
            .route("/users/:id", get(users::get_user))
    )
}
```

---

# Parte 3 — Conectar el módulo a la API

## Paso 17 — `main.rs` funcional

```rust
// apps/api/src/main.rs
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Cargar .env.local en desarrollo
    let _ = dotenvy::from_filename(".env.local");

    // Inicializar logs
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Configuración con fail-fast
    let config = infrastructure::config::AppConfig::load()
        .expect("❌ Configuración inválida — revisa las variables de entorno");

    // Validar PASETO_SECRET al arrancar
    assert!(
        config.paseto_secret.len() == 32,
        "❌ PASETO_SECRET debe tener exactamente 32 bytes"
    );

    // Pool de DB
    let pool = Arc::new(database::create_pool(&config.database_url).await?);

    // Migraciones automáticas al arrancar
    sqlx::migrate!("../../data/migrations").run(&*pool).await?;
    tracing::info!("migraciones ejecutadas");

    // AppState
    let state = infrastructure::http::state::AppState {
        user_repo: Arc::new(database::repositories::SqliteUserRepository::new(pool)),
        config:    Arc::new(config.clone()),
    };

    // Router
    let app = infrastructure::http::router::api_router()
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(port = config.server_port, "servidor iniciado");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async { signal::ctrl_c().await.expect("failed to install Ctrl+C handler") };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv().await;
    };
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
```

```bash
just dev-api
# → servidor iniciado en puerto 8080

# Probar en otra terminal
curl http://localhost:8080/api/v1/users
# → []

curl -X POST http://localhost:8080/api/v1/users \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
# → 201 Created { "id": "...", "email": "test@example.com" }
```

---

# Parte 4 — Tests por capa (ADR 0009)

## Paso 18 — Capa 1: Tests de dominio (sin deps)

```rust
// crates/domain/src/value_objects/email.rs — al final
#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn email_valido()               { assert!(Email::new("user@example.com").is_ok()); }
    #[test] fn email_a_minusculas()         { assert_eq!(Email::new("User@EXAMPLE.COM").unwrap().as_str(), "user@example.com"); }
    #[test] fn email_con_subdominio()       { assert!(Email::new("u@mail.example.co.uk").is_ok()); }
    #[test] fn email_vacio_invalido()       { assert!(Email::new("").is_err()); }
    #[test] fn email_sin_arroba_invalido()  { assert!(matches!(Email::new("userexample.com").unwrap_err(), crate::errors::DomainError::InvalidEmail(_))); }
    #[test] fn email_sin_dominio_invalido() { assert!(Email::new("user@").is_err()); }
    #[test] fn email_dominio_sin_punto()    { assert!(Email::new("user@localhost").is_err()); }
    #[test] fn email_trim_espacios()        { assert_eq!(Email::new("  user@example.com  ").unwrap().as_str(), "user@example.com"); }
}
```

```bash
cargo nextest run -p domain
# Todos deben pasar en <50ms
```

## Paso 19 — Capa 2: Tests de application (con mockall)

```rust
// crates/application/src/use_cases/users/create_user.rs — al final
#[cfg(test)]
mod tests {
    use super::*;
    use domain::{User, UserId, Email, PasswordHash};
    use mockall::{mock, predicate::*};

    mock! {
        Repo {}
        impl UserRepository for Repo {
            async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;
            async fn save(&self, user: &User)                   -> Result<(), DomainError>;
            async fn find_by_id(&self, id: &UserId)            -> Result<Option<User>, DomainError>;
            async fn soft_delete(&self, id: &UserId)           -> Result<(), DomainError>;
            async fn list(&self, l: i64, o: i64)               -> Result<Vec<User>, DomainError>;
            async fn count(&self)                              -> Result<i64, DomainError>;
            async fn has_permission(&self, id: &UserId, p: &str) -> Result<bool, DomainError>;
        }
    }

    fn fake_hash() -> Arc<dyn Fn(&str) -> Result<String, DomainError> + Send + Sync> {
        Arc::new(|_| Ok("hashed".into()))
    }

    #[tokio::test]
    async fn email_nuevo_crea_usuario() {
        let mut mock = MockRepo::new();
        mock.expect_find_active_by_email().once().returning(|_| Ok(None));
        mock.expect_save().once().returning(|_| Ok(()));

        let result = CreateUserUseCase { repo: Arc::new(mock), hash_fn: fake_hash() }
            .execute(CreateUserInput { email: "nuevo@example.com".into(), password: "pass".into() })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "nuevo@example.com");
    }

    #[tokio::test]
    async fn email_duplicado_falla_sin_llamar_save() {
        let mut mock = MockRepo::new();
        let user = User::new(Email::new("x@example.com").unwrap(), PasswordHash::from_hash("h".into()));
        mock.expect_find_active_by_email().once().returning(move |_| Ok(Some(user.clone())));
        mock.expect_save().never();

        let result = CreateUserUseCase { repo: Arc::new(mock), hash_fn: fake_hash() }
            .execute(CreateUserInput { email: "x@example.com".into(), password: "pass".into() })
            .await;

        assert!(matches!(result.unwrap_err(), DomainError::EmailAlreadyExists));
    }

    #[tokio::test]
    async fn email_invalido_no_toca_la_db() {
        let mut mock = MockRepo::new();
        mock.expect_find_active_by_email().never();
        mock.expect_save().never();

        let result = CreateUserUseCase { repo: Arc::new(mock), hash_fn: fake_hash() }
            .execute(CreateUserInput { email: "no-es-email".into(), password: "pass".into() })
            .await;

        assert!(matches!(result.unwrap_err(), DomainError::InvalidEmail(_)));
    }
}
```

```bash
cargo nextest run -p application
```

## Paso 20 — Capa 3: Tests de integración con SQLite real

```rust
// crates/database/tests/user_repository_test.rs
use std::sync::Arc;
use domain::{User, Email, PasswordHash, UserRepository};
use database::{create_pool, repositories::SqliteUserRepository};

async fn setup() -> SqliteUserRepository {
    let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!("../../data/migrations").run(&pool).await.unwrap();
    SqliteUserRepository::new(Arc::new(pool))
}

fn fake_user(email: &str) -> User {
    User::new(Email::new(email).unwrap(), PasswordHash::from_hash("hash".into()))
}

#[tokio::test]
async fn guardar_y_recuperar_por_email() {
    let repo = setup().await;
    let user = fake_user("test@example.com");
    repo.save(&user).await.unwrap();
    let found = repo.find_active_by_email(&Email::new("test@example.com").unwrap()).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email.as_str(), "test@example.com");
}

#[tokio::test]
async fn soft_delete_oculta_el_usuario() {
    let repo = setup().await;
    let user = fake_user("delete@example.com");
    let id   = user.id.clone();
    repo.save(&user).await.unwrap();
    repo.soft_delete(&id).await.unwrap();
    let found = repo.find_active_by_email(&Email::new("delete@example.com").unwrap()).await.unwrap();
    assert!(found.is_none()); // No aparece en búsquedas activas
}

#[tokio::test]
async fn email_puede_reutilizarse_tras_soft_delete() {
    let repo  = setup().await;
    let user1 = fake_user("reuse@example.com");
    let id1   = user1.id.clone();
    repo.save(&user1).await.unwrap();
    repo.soft_delete(&id1).await.unwrap();

    // Mismo email, usuario nuevo — el índice parcial lo permite
    let user2 = fake_user("reuse@example.com");
    assert!(repo.save(&user2).await.is_ok());
}

#[tokio::test]
async fn has_permission_con_rol_admin() {
    let repo = setup().await;
    // El seed crea admin con todos los permisos
    let admin = repo.find_active_by_email(
        &Email::new("admin@admin.com").unwrap()
    ).await.unwrap().unwrap();
    assert!(repo.has_permission(&admin.id, "users:read").await.unwrap());
    assert!(repo.has_permission(&admin.id, "users:write").await.unwrap());
}

#[tokio::test]
async fn paginacion_funciona() {
    let repo = setup().await;
    for i in 0..5 {
        repo.save(&fake_user(&format!("user{}@example.com", i))).await.unwrap();
    }
    let page1 = repo.list(2, 0).await.unwrap();
    let page2 = repo.list(2, 2).await.unwrap();
    assert_eq!(page1.len(), 2);
    assert!(page2.len() >= 2); // El seed también tiene usuarios
}
```

```bash
cargo nextest run -p database
```

---

# Parte 5 — El flujo completo de una petición

## Lo que acabas de construir

```
POST /api/v1/users   { "email": "nuevo@example.com", "password": "abc123" }

① apps/api — main.rs recibe la petición en el router

② crates/infrastructure — create_user handler
   → extrae Json<CreateUserRequest>
   → construye CreateUserUseCase con hash_fn de crates/auth
   → delega — sin lógica de negocio aquí

③ crates/application — CreateUserUseCase.execute()
   → Email::new() valida el formato                    [domain]
   → repo.find_active_by_email()                       [port → database]
   → hash_password(argon2id)                           [auth]
   → User::new(email, password_hash)                   [domain]
   → repo.save(&user)                                  [port → database]

④ crates/domain — Email::new() valida, User::new() crea entidad

⑤ crates/database — SqliteUserRepository
   → INSERT INTO users ... (índice parcial garantiza unicidad en activos)

← 201 Created   { "id": "...", "email": "nuevo@example.com" }
```

## El mismo patrón para el siguiente feature: Auth

```
1. crates/domain    → value_object: SessionToken
2. crates/domain    → port: SessionRepository, AuditRepository
3. crates/database  → model: SessionRow, AuditRow
4. crates/database  → repository: SqliteSessionRepository, SqliteAuditRepository
5. crates/auth      → PasetoService ya está listo
6. crates/application → use_case: login.rs, logout.rs, refresh.rs
7. crates/infrastructure → handlers/auth.rs + middleware/auth.rs
8. Tests → mismo orden: domain → application → database
```

La arquitectura hexagonal hace que cada módulo nuevo sea predecible.
Aprendes el patrón una vez — lo repites para cada feature.

---

## Checklist de lo que deberías tener al terminar esta guía

```
[x] Workspace con 10 crates compilando limpio
[x] 6 migraciones ejecutadas — RBAC + Sessions + Audit
[x] Value objects: UserId, Email, PasswordHash con validación
[x] Entity User con Soft Delete (is_active, soft_delete)
[x] Ports: UserRepository con has_permission y find_active_by_email
[x] DomainError + AppError con IntoResponse — formato JSON consistente
[x] Pool SQLite con WAL y PRAGMAs optimizados
[x] SqliteUserRepository con has_permission (JOIN 4 tablas)
[x] argon2id en crates/auth (hash + verify)
[x] PasetoService en crates/auth con validación de 32 bytes
[x] Use cases: create_user, get_user
[x] AppState + handlers + router en infrastructure
[x] main.rs con fail-fast en config y migraciones automáticas
[x] Tests de dominio sin deps — email, password
[x] Tests de application con mockall
[x] Tests de integración con SQLite :memory: — incluyendo has_permission y soft delete
[x] Servidor respondiendo JSON correcto en /api/v1/users
```

---

## Qué construir después

1. **Auth completa** — endpoints register/login/logout/refresh + auth_middleware + RBAC (Bloque III del ROADMAP)
2. **Documentación** — utoipa en los handlers + Scalar en `/docs` (Bloque IV)
3. **Jobs y email** — EmailJob con Apalis + ResendMailer (Bloque V)
4. **Frontend** — Astro + TanStack Query consumiendo los endpoints (Bloque VII)
5. **Deploy** — Containerfile distroless + Kamal en VPS de $5 (Bloque VIII)
