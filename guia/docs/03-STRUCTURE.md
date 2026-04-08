# Estructura del Monorepo — boilerplate

> Arquitectura usada por proyectos Rust grandes en producción real:
> **Tauri**, **Fly.io**, **Zed Editor**, **TiKV**
>
> Idea central: crates pequeños con `Cargo.toml` propio — las fronteras de arquitectura
> las hace cumplir el compilador, no las convenciones.

---

## Árbol completo

```
boilerplate/
│
├── Cargo.toml                   # Workspace root — declara miembros y versiones compartidas
├── Cargo.lock                   # Lockfile — siempre en git (es un binario)
├── deny.toml                    # cargo-deny: licencias + CVEs — ADR 0010
├── justfile                     # Comandos del proyecto — ADR 0012
├── lefthook.yml                 # Git hooks: fmt + lint — ADR 0012
├── .env.example                 # Contrato de configuración — ADR 0002
├── pnpm-workspace.yaml
├── README.md
│
├── apps/                        # Binarios — consumen los crates, sin lógica propia
│   ├── api/                     # Servidor Axum HTTP — ADR 0003
│   ├── web/                     # Frontend Astro + Svelte 5 — ADR 0022
│   ├── mailer/                  # Plantillas React Email — ADR 0019
│   │   └── emails/
│   │       ├── welcome.tsx
│   │       ├── password_reset.tsx
│   │       ├── lead_welcome.tsx
│   │       └── notification.tsx
│   ├── desktop/                 # Tauri 2.0 — Fase 1 — ADR 0030
│   └── cli/                     # Sintonía CLI — scaffolding — ADR 0028
│
├── crates/                      # El corazón del sistema — lógica de negocio
│   ├── domain/                  # Núcleo puro — sin dependencias externas — ADR 0001
│   ├── application/             # Casos de uso — orquesta dominio — ADR 0001
│   ├── infrastructure/          # HTTP handlers, config, servicios externos — ADR 0003
│   ├── database/                # Repositorios SQLx + pool + Moka — ADR 0004, 0017
│   ├── auth/                    # argon2id + PASETO v4 — ADR 0008
│   ├── mailer/                  # Puerto Mailer + adaptador Resend — ADR 0019
│   ├── storage/                 # Puerto Storage + adaptador Tigris — ADR 0020
│   └── events/                  # NATS JetStream — Fase 2 — ADR 0025
│
├── proto/                       # Contratos Protobuf (Fase 2) — ADR 0027
│   ├── buf.yaml
│   ├── buf.gen.yaml
│   ├── user/v1/user.proto
│   └── common/v1/common.proto
│
├── data/
│   ├── migrations/              # SQLx migrations versionadas — ADR 0005, 0006
│   │   ├── 20260305135148_create_users_table.sql
│   │   ├── 20260305135149_create_rbac.sql
│   │   ├── 20260305135150_create_tokens.sql
│   │   ├── 20260305135151_create_audit_logs.sql
│   │   ├── 20260305135152_seed_system_data.sql
│   │   └── 20260305135153_create_sessions.sql
│   └── seeds/                   # Seeds de desarrollo adicionales
│
├── infra/                       # Infraestructura como código — ADR 0014
│   ├── docker/
│   │   ├── Containerfile        # Multi-stage + distroless (~10MB final) — ADR 0013
│   │   └── compose.yml
│   ├── caddy/
│   │   └── Caddyfile
│   ├── litestream/
│   │   └── litestream.yml
│   └── kamal/
│       └── deploy.yml
│
└── docs/
    ├── ARCHITECTURE.md          # Flujos y decisiones de arquitectura
    ├── STRUCTURE.md             # Este archivo
    ├── STACK.md                 # Stack completo con versiones
    ├── ROADMAP.md               # Bloques de trabajo y estado
    ├── TODO.md                  # Progreso real por tarea
    ├── SINTONIA-CLI.md          # Referencia técnica del CLI
    ├── DASHBOARD-DISENO.md      # Guía de diseño del dashboard
    └── adr/
        ├── 0001 Arquitectura Hexagonal
        ├── 0002 Configuración Tipeada
        ├── ...
        ├── 0031 Escalamiento
        ├── future/
        │   ├── ADR-F001-surrealdb-rocksdb.md
        │   └── ADR-F002-postgresql-escala-horizontal.md
        └── BRUJULA-ADR-resumen.md
```

---

## Workspace root — `Cargo.toml`

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

[profile.release]
opt-level     = "z"     # Optimizar por tamaño — ADR 0013
lto           = true
codegen-units = 1
panic         = "abort"
strip         = true

[workspace.dependencies]
# Runtime
tokio      = { version = "1",   features = ["full"] }
serde      = { version = "1",   features = ["derive"] }
serde_json = "1"
uuid       = { version = "1",   features = ["v4", "v7", "serde"] }
time       = { version = "0.3", features = ["serde", "macros"] }

# Errores — ADR 0007
thiserror = "2"
anyhow    = "1"

# DB — ADR 0004
sqlx = { version = "0.8", features = [
    "sqlite", "runtime-tokio", "macros", "uuid", "time"
] }

# Web — ADR 0003
axum           = { version = "0.8", features = ["macros"] }
axum-extra     = { version = "0.9", features = ["cookie", "typed-header"] }
tower          = "0.5"
tower-http     = { version = "0.6", features = [
    "cors", "compression-gzip", "compression-br",
    "trace", "timeout", "request-id"
] }
tower-governor = "0.4"   # ADR 0009

# Auth — ADR 0008
argon2   = "0.5"
pasetors = { version = "0.7", features = ["v4"] }

# Cache y jobs — ADR 0017, 0018
moka       = { version = "0.12", features = ["future"] }
apalis     = { version = "0.6",  features = ["sqlite", "tracing", "retry"] }
apalis-sql = { version = "0.6",  features = ["sqlite"] }

# Observabilidad — ADR 0016
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
sentry             = "0.34"

# Docs API — ADR 0021
utoipa        = { version = "4", features = ["axum_extras", "uuid"] }
utoipa-scalar = { version = "0.1", features = ["axum"] }

# Config — ADR 0002
config  = "0.14"
dotenvy = "0.15"

# Email — ADR 0019
resend-rs = "0.5"

# Storage — ADR 0020
aws-config = "1.1"
aws-sdk-s3 = "1.1"

# Eventos (Fase 2) — ADR 0025
async-nats = "0.36"

[workspace.dev-dependencies]
tokio    = { version = "1", features = ["test", "macros"] }
mockall  = "0.13"
reqwest  = { version = "0.12", features = ["json"] }
httpmock = "0.7"
```

---

## `crates/domain/` — Núcleo puro — ADR 0001

**Regla absoluta:** solo `thiserror`, `uuid`, `time` y `serde`. Si `sqlx` o `axum`
aparecen aquí, la arquitectura está rota. El compilador lo hace cumplir.

```
crates/domain/
├── Cargo.toml               # thiserror, uuid, time, serde — nada más
└── src/
    ├── lib.rs
    ├── entities/
    │   ├── user.rs          # struct User — Soft Delete + is_active()
    │   ├── role.rs
    │   ├── session.rs
    │   ├── audit_log.rs
    │   └── lead.rs          # Para captura de leads de la landing — ADR 0029
    ├── value_objects/
    │   ├── user_id.rs       # newtype UserId(Uuid)
    │   ├── email.rs         # Email — validado en construcción, normalizado a minúsculas
    │   ├── password_hash.rs # PasswordHash — nunca expone el hash
    │   └── permission.rs    # Permission — formato "recurso:acción"
    ├── ports/               # Traits que la infraestructura implementa
    │   ├── user_repository.rs       # find_active_by_email, has_permission, soft_delete
    │   ├── role_repository.rs
    │   ├── session_repository.rs
    │   ├── audit_repository.rs      # Insert-only
    │   ├── token_repository.rs      # create, use_token, cleanup_expired
    │   ├── lead_repository.rs       # ADR 0029
    │   ├── mailer.rs
    │   └── storage_repository.rs
    └── errors/
        ├── domain_error.rs  # DomainError — reglas de negocio violadas
        └── app_error.rs     # AppError + IntoResponse — ADR 0007
```

```toml
# crates/domain/Cargo.toml — LÍMITE DURO
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
```

---

## `crates/application/` — Casos de uso — ADR 0001

Orquesta el dominio usando los ports. Sin SQL, sin HTTP, sin lógica de infraestructura.

```
crates/application/
├── Cargo.toml               # domain + tracing + uuid
└── src/
    ├── lib.rs
    └── use_cases/
        ├── auth/
        │   ├── register.rs       # Email → argon2id → save → EmailJob
        │   ├── login.rs          # verify → PASETO 15min → refresh → session → audit
        │   ├── logout.rs         # revocar session + refresh token
        │   └── refresh_token.rs
        ├── users/
        │   ├── get_user.rs
        │   ├── list_users.rs
        │   ├── update_user.rs
        │   └── soft_delete_user.rs
        ├── roles/
        │   └── check_permission.rs
        └── leads/
            └── capture_lead.rs   # valida, dedup silenciosa, save, LeadWelcomeJob — ADR 0029
```

---

## `crates/infrastructure/` — HTTP y configuración — ADR 0003

Los handlers delegan inmediatamente al caso de uso. Sin lógica de negocio.

```
crates/infrastructure/
├── Cargo.toml               # application + domain + axum + tower + config
└── src/
    ├── lib.rs
    ├── http/
    │   ├── router.rs            # Router modular + docs_router()
    │   ├── state.rs             # AppState — composition root
    │   ├── middleware/
    │   │   ├── auth.rs          # Extrae PASETO → UserId en Extensions
    │   │   ├── rate_limit.rs    # tower-governor — ADR 0009
    │   │   └── audit.rs         # Registra acción en audit_logs post-response
    │   └── handlers/
    │       ├── health.rs        # GET /health — verifica DB
    │       ├── auth.rs          # register, login, refresh, logout
    │       ├── users.rs         # CRUD con RBAC
    │       └── leads.rs         # POST /api/v1/leads — ADR 0029
    ├── config/
    │   └── app_config.rs        # AppConfig — fail-fast — ADR 0002
    └── docs.rs                  # ApiDoc + Scalar — ADR 0021
```

---

## `crates/database/` — Repositorios SQLx — ADR 0004

La única crate que puede importar `sqlx`. SQL explícito chequeado en compile-time.

```
crates/database/
├── Cargo.toml               # domain + sqlx + moka + uuid + time
└── src/
    ├── lib.rs
    ├── pool.rs              # create_pool() con WAL + PRAGMAs optimizados
    ├── repositories/
    │   ├── sqlite_user_repository.rs      # impl UserRepository + has_permission (JOIN 4 tablas)
    │   ├── cached_user_repository.rs      # Decorator Moka — ADR 0017
    │   ├── sqlite_role_repository.rs
    │   ├── sqlite_session_repository.rs
    │   ├── sqlite_audit_repository.rs     # Insert-only
    │   ├── sqlite_token_repository.rs
    │   └── sqlite_lead_repository.rs      # ADR 0029
    └── models/
        ├── user_row.rs      # Struct de mapeo DB — separado de la entidad de dominio
        ├── role_row.rs
        ├── session_row.rs
        ├── audit_row.rs
        ├── token_row.rs
        └── lead_row.rs
```

---

## `crates/auth/` — Autenticación — ADR 0008

argon2id para passwords. PASETO v4 Local para access tokens. Tokens opacos para refresh.

```
crates/auth/
├── Cargo.toml               # domain + argon2 + pasetors
└── src/
    ├── password.rs          # hash_password + verify_password (argon2id, parámetros OWASP)
    ├── paseto.rs            # PasetoService: generate(15min) + verify — v4.local.xxx
    └── token.rs             # generate_opaque_token() — 32 bytes aleatorios
```

---

## `crates/mailer/` — Email — ADR 0019

Puerto abstracto — el dominio no conoce el proveedor.

```
crates/mailer/
├── Cargo.toml               # domain + resend-rs
└── src/
    ├── log_mailer.rs        # Imprime en tracing::info! — solo development
    └── resend_mailer.rs     # impl Mailer con Resend API
```

---

## `crates/storage/` — Almacenamiento — ADR 0020

Puerto abstracto `StorageRepository`. Adaptador Tigris S3.

```
crates/storage/
├── Cargo.toml               # domain + aws-config + aws-sdk-s3
└── src/
    ├── client.rs
    └── tigris_repository.rs # upload, get_presigned_url, delete
```

---

## `crates/events/` — Bus de eventos — Fase 2 — ADR 0025, 0026

Activar cuando existan módulos que necesiten comunicarse asincrónicamente.

```
crates/events/
├── Cargo.toml               # domain + async-nats
└── src/
    ├── subjects.rs          # Constantes: boilerplate.{dominio}.{evento}.v1
    ├── publisher.rs         # publish_event<T: Serialize>() con ACK
    └── consumers/
        └── user_consumer.rs
```

---

## `apps/api/` — Servidor Axum — ADR 0003

Ensambla todo. Sin lógica de negocio — solo composición.

```
apps/api/
├── Cargo.toml               # infrastructure + database + auth + mailer + storage + tokio
└── src/
    ├── main.rs              # load config → telemetry → pool → migrate → state → serve
    ├── setup.rs             # build_state() — composition root único
    ├── telemetry.rs         # tracing JSON + Sentry — ADR 0016
    └── jobs/
        ├── worker.rs        # start_workers() — Apalis workers — ADR 0018
        ├── email_job.rs
        └── cleanup_job.rs
```

```rust
// apps/api/src/setup.rs — único lugar donde se ensamblan las dependencias
pub async fn build_state(pool: SqlitePool, config: &AppConfig) -> AppState {
    let user_db     = SqliteUserRepository::new(pool.clone());
    let user_cached = CachedUserRepository::new(user_db); // Decorator Moka

    AppState {
        user_repo:    Arc::new(user_cached),
        role_repo:    Arc::new(SqliteRoleRepository::new(pool.clone())),
        session_repo: Arc::new(SqliteSessionRepository::new(pool.clone())),
        audit_repo:   Arc::new(SqliteAuditRepository::new(pool.clone())),
        token_repo:   Arc::new(SqliteTokenRepository::new(pool.clone())),
        lead_repo:    Arc::new(SqliteLeadRepository::new(pool.clone())),
        paseto:       Arc::new(PasetoService::new(&config.paseto_secret)),
        mailer:       Arc::new(build_mailer(config)),
        storage:      Arc::new(TigrisRepository::new(config)),
        pool,
        config:       Arc::new(config.clone()),
    }
}
```

---

## `apps/web/` — Frontend Astro + Svelte 5 — ADR 0022

HTML primero, JavaScript solo donde sea necesario.

```
apps/web/
├── package.json
├── astro.config.mjs
└── src/
    ├── pages/
    │   ├── index.astro          # Landing page — ADR 0029
    │   ├── login.astro
    │   ├── register.astro
    │   └── dashboard/
    │       ├── index.astro      # KPIs + gráfico + feed de eventos
    │       ├── users/
    │       │   ├── index.astro  # Tabla paginada — requiere users:read
    │       │   └── [id].astro
    │       └── audit/
    │           └── index.astro  # requiere audit:read
    ├── components/
    │   ├── ui/                  # shadcn-svelte + Bits UI
    │   │   ├── EmptyState.svelte
    │   │   ├── PermissionGate.svelte
    │   │   └── ThemeToggle.svelte
    │   ├── layout/
    │   │   ├── Sidebar.svelte
    │   │   ├── Topbar.svelte
    │   │   └── CommandPalette.svelte
    │   ├── dashboard/
    │   │   ├── KpiCard.svelte
    │   │   ├── ActivityChart.svelte
    │   │   ├── EventFeed.svelte
    │   │   └── SystemHealth.svelte
    │   ├── landing/             # ADR 0029
    │   │   └── LeadForm.svelte
    │   └── users/
    │       └── UserTable.svelte
    ├── lib/
    │   ├── api/
    │   │   ├── client.ts        # fetch base con auth headers
    │   │   ├── auth.ts
    │   │   ├── users.ts
    │   │   └── leads.ts
    │   ├── types/
    │   │   └── api.ts           # GENERADO por buf generate — no editar manualmente
    │   ├── stores/
    │   │   └── auth.svelte.ts   # $state Runes — user, accessToken, isLoggedIn
    │   └── validation/
    │       └── schemas.ts       # ArkType schemas
    ├── styles/
    │   └── global.css           # CSS variables de marca — tokens compartidos
    ├── messages/                # Paraglide JS — editar manualmente — ADR 0023
    │   ├── es.json
    │   └── en.json
    └── layouts/
        ├── BaseLayout.astro
        ├── LandingLayout.astro  # ADR 0029
        └── DashboardLayout.astro # verifica PASETO en servidor
```

---

## Jerarquía de dependencias entre crates

```
crates/domain          ← sin dependencias externas
    ↑
crates/application     ← solo domain
    ↑
crates/database        ← domain + sqlx
crates/auth            ← domain + argon2 + pasetors
crates/mailer          ← domain + resend-rs
crates/storage         ← domain + aws-sdk-s3
crates/events          ← domain + async-nats
    ↑
crates/infrastructure  ← application + axum + config + todos los crates anteriores
    ↑
apps/api               ← infrastructure + database + auth + mailer + storage
```

**La regla:** las flechas van hacia arriba. Ningún crate puede importar uno que esté
por encima de él en la jerarquía. El compilador lo hace cumplir a través de `Cargo.toml`.

---

## Reglas arquitectónicas

| # | Regla | Garantizada por | ADR |
|---|-------|----------------|-----|
| 1 | `crates/domain` no importa infraestructura | `Cargo.toml` domain | 0001 |
| 2 | SQL solo en `crates/database` | `Cargo.toml` domain/application | 0001 |
| 3 | Los handlers delegan inmediatamente al caso de uso | Code review | 0001 |
| 4 | JWT prohibido — solo PASETO v4 Local | `jsonwebtoken` no en workspace | 0008 |
| 5 | Tipos TypeScript generados por `buf generate` | CI verifica diff en `api.ts` | 0027 |
| 6 | `Row` struct separado de cada entidad | Convenio en `crates/database/src/models/` | 0004 |
| 7 | `async fn` en ports — Rust 2024 nativo | PRs de domain | 0001 |
| 8 | Imagen final distroless — ~10MB, sin shell | ADR 0013 | 0013 |
| 9 | Soft Delete en users — nunca `DELETE` real | Trigger `deleted_at` | 0006 |
| 10 | Toda acción autenticada se audita | Middleware `audit.rs` automático | 0006 |
