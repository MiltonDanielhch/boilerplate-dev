# Stack Definitivo — boilerplate

> **Rust Workspace · Arquitectura Hexagonal · Ejecutable desde el día uno · Escala sin reescribir**

---

## Principios clave

| # | Principio |
|---|-----------|
| 01 | Primero funciona → luego escala |
| 02 | Menos moving parts = más velocidad |
| 03 | El compilador hace cumplir la arquitectura — no las convenciones |
| 04 | Todo lo avanzado es opcional y gradual |
| 05 | Domain sin dependencias, siempre |
| 06 | SQL explícito, sin ORM pesado |
| 07 | JWT prohibido — solo PASETO v4 Local |
| 08 | Soft Delete en users — nunca DELETE real |

---

## Arquitectura base

```
Frontend (Astro SSR + Svelte 5)
        ↓
HTTP API — tipos TypeScript via buf generate (ConnectRPC — Fase 2)
        ↓
Axum 0.8 — tower · CORS · compression · tracing · tower-governor
        ↓
Application — use cases, orquesta dominio, sin lógica de infra
        ↓
Domain — núcleo puro (Entities · Value Objects · Ports)
        ↓
Infrastructure — SQLx repos · HTTP handlers · Auth · Jobs · Cache
        ↓
SQLite WAL → Litestream → S3/Tigris
```

---

## Estructura de crates

```
boilerplate/
├── apps/
│   ├── api/         # Binario — ensambla todo, sin lógica propia
│   ├── web/         # Astro SSR + Svelte 5
│   └── cli/         # Sintonía CLI (Fase 2) — ver ADR 0028
│
├── crates/
│   ├── domain/      # Cargo.toml: thiserror, uuid, time, serde — NADA MÁS
│   ├── application/ # Cargo.toml: solo domain
│   ├── infrastructure/ # HTTP, config, router
│   ├── database/    # El único crate con sqlx — pool + repositorios + Moka
│   ├── auth/        # argon2id + PASETO v4
│   ├── mailer/      # Puerto Mailer + adaptador Resend
│   ├── storage/     # Puerto Storage + adaptador Tigris/S3
│   └── events/      # NATS JetStream (Fase 2)
│
├── data/migrations/ # 6 migraciones: RBAC + Sessions + Audit — ver ADR 0006
├── infra/           # docker · caddy · litestream · kamal — ver ADR 0014
└── proto/           # Protobuf (Fase 2 — ConnectRPC) — ver ADR 0027
```

**Por qué `crates/` con Cargo.toml por crate:** si `crates/domain/Cargo.toml` no declara
`sqlx`, es imposible importarlo desde domain. La arquitectura hexagonal queda garantizada
por el compilador, no por code review. — ver ADR 0001

---

## 1. Backend core

### Lenguaje y runtime

```toml
rust  = "edition 2024"
tokio = { version = "1", features = ["full"] }
```

### Serialización y tipos

```toml
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
uuid       = { version = "1", features = ["v4", "v7", "serde"] }
time       = { version = "0.3", features = ["serde", "macros", "formatting", "parsing"] }
```

### Manejo de errores — ADR 0007

```toml
thiserror = "2"   # Domain — errores tipados con variantes explícitas
anyhow    = "1"   # Application/Infra — propagación sin boilerplate
```

Jerarquía: `DomainError → AppError → IntoResponse (HTTP)`

Formato de respuesta consistente en toda la API:
```json
{ "error": "email_already_exists", "message": "el email ya está registrado" }
```

Los errores 500 nunca exponen detalles al cliente — `tracing::error!` internamente.

### Configuración — fail-fast — ADR 0002

```toml
config  = "0.14"
dotenvy = "0.15"
```

```rust
// Si falta cualquier variable, el proceso no arranca — antes de aceptar tráfico
let config = AppConfig::load().expect("❌ Configuración inválida");
assert!(config.paseto_secret.len() == 32, "❌ PASETO_SECRET debe tener 32 bytes");
```

---

## 2. Web API — Axum 0.8 — ADR 0003

```toml
axum           = { version = "0.8", features = ["macros"] }
axum-extra     = { version = "0.9", features = ["cookie", "typed-header"] }
tower          = "0.5"
tower-http     = { version = "0.6", features = [
    "cors", "compression-gzip", "compression-br",
    "trace", "timeout", "request-id"
] }
tower-governor = "0.4"
```

### Middleware en orden

| # | Middleware | Crate | ADR |
|---|-----------|-------|-----|
| 1 | SetRequestIdLayer | `tower-http` | 0003 |
| 2 | TraceLayer con request_id | `tower-http` | 0016 |
| 3 | CompressionLayer (gzip + br) | `tower-http` | 0003 |
| 4 | CorsLayer | `tower-http` | 0003 |
| 5 | TimeoutLayer (30s) | `tower-http` | 0003 |
| 6 | Rate limit global — 10 req/s, burst 30 | `tower-governor` | 0009 |
| 7 | Rate limit auth — 1 req/s, burst 5 en `/auth/*` | `tower-governor` | 0009 |
| 8 | auth_middleware (PASETO) | custom | 0008 |
| 9 | permission_middleware (RBAC) | custom | 0006 |
| 10 | audit_middleware (post-response) | custom | 0006 |

---

## 3. Base de datos — ADR 0004, 0006

```toml
sqlx = { version = "0.8", features = [
    "sqlite", "runtime-tokio", "macros", "uuid", "time"
] }
```

### Arquitectura

```
SQLx (compile-time checked queries)
        ↓
SQLite WAL mode + PRAGMAs optimizados
  journal_mode=WAL · synchronous=NORMAL · mmap_size=30GB · cache_size=64MB
        ↓
Litestream (replicación continua — RPO ~1 segundo)
        ↓
S3 / Tigris (backup)
```

### 6 migraciones del sistema — ADR 0006

| # | Migración | Tablas |
|---|-----------|--------|
| 1 | `create_users_table` | `users` (Soft Delete + trigger), `user_roles` |
| 2 | `create_rbac` | `roles`, `permissions`, `role_permissions` |
| 3 | `create_tokens` | `tokens` (verificación email + reset) |
| 4 | `create_audit_logs` | `audit_logs` (ON DELETE SET NULL) |
| 5 | `seed_system_data` | Admin user + roles + permisos base |
| 6 | `create_sessions` | `sessions` (IP + UA + expiry + trigger) |

### Reglas inamovibles

- `sqlx::query_as!` solo en `crates/database/` — el Cargo.toml lo garantiza
- Soft Delete en users — `UPDATE deleted_at`, nunca `DELETE`
- Índice parcial en email — permite reutilizar emails de cuentas borradas
- RBAC N:M — usuarios ↔ roles ↔ permisos con tablas intermedias

### Path de escala — ADR 0031

```
SQLite WAL (hasta ~100 writes/s)
    ↓ Nivel 2a: >100 writes/s sostenidos
Turso (SQLite distribuido — misma API SQLx)    → ADR F-001
    ↓ Nivel 2b: joins analíticos o réplicas
PostgreSQL                                      → ADR F-002
```

---

## 4. Auth — PASETO v4 + argon2id — ADR 0008

```toml
argon2   = "0.5"
pasetors = { version = "0.7", features = ["v4"] }
# jsonwebtoken NO está en el workspace — PROHIBIDO
```

### Por qué PASETO en lugar de JWT

| JWT | PASETO v4 Local |
|-----|-----------------|
| Header `alg` manipulable (`alg: none`) | Sin header de algoritmo — XChaCha20-Poly1305 fijo |
| Payload visible en Base64 | Payload **cifrado** — user_id invisible para intermediarios |
| Múltiples algoritmos = superficie de ataque | Un solo algoritmo = imposible degradar |

### Flujo de autenticación

```
POST /auth/register → argon2id hash → save → EmailJob (Apalis)
POST /auth/login    → verify argon2id → PASETO 15min → refresh token → session (IP + UA)
POST /auth/refresh  → verify refresh → nuevo PASETO + nuevo refresh (rotación obligatoria)
POST /auth/logout   → revocar session + refresh token en DB
```

### RBAC — permisos en formato `"recurso:acción"` — ADR 0006

```rust
// Uso declarativo — el middleware hace el JOIN de 4 tablas automáticamente
.route("/api/v1/users", get(list_users).layer(require_permission("users:read")))
.route("/api/v1/users", post(create_user).layer(require_permission("users:write")))
```

Permisos del seed: `users:read`, `users:write`, `audit:read`, `roles:read`, `roles:write`

---

## 5. Documentación API — ADR 0021

```toml
utoipa        = { version = "4", features = ["axum_extras", "uuid"] }
utoipa-scalar = { version = "0.1", features = ["axum"] }
```

- Macros `#[utoipa::path]` solo en DTOs y handlers — nunca en entidades de dominio
- `/docs` — Scalar UI (solo en development y staging)
- `/openapi.json` — disponible en producción (para IAs y herramientas externas)
- IA-ready: Windsurf y GPT pueden entender la API al instante desde el spec

---

## 6. Caché — Moka Decorator — ADR 0017

```toml
moka = { version = "0.12", features = ["future"] }
```

```rust
// Patrón Decorator — el dominio no sabe que existe el caché
let db_repo     = SqliteUserRepository::new(pool);
let cached_repo = CachedUserRepository::new(db_repo);  // TTL 5min, max 10k
AppState { user_repo: Arc::new(cached_repo), ... }
```

Regla crítica: `cache.invalidate()` en todo método de escritura — sin esto hay datos obsoletos.
Límite: 100MB RAM total para todos los cachés del sistema.

> Moka es in-process. Si escala a múltiples instancias → agregar Redis como L2.
> El port `UserRepository` no cambia — solo la implementación.

---

## 7. Jobs asíncronos — Apalis + SQLite — ADR 0018

```toml
apalis     = { version = "0.6", features = ["sqlite", "tracing", "retry"] }
apalis-sql = { version = "0.6", features = ["sqlite"] }
```

Sin infraestructura extra en MVP — misma SQLite del sistema.
Durabilidad garantizada: los jobs sobreviven reinicios del proceso.

```rust
// Jobs del sistema
EmailJob    // welcome, password_reset, lead_welcome, notification → via Resend (ADR 0019)
CleanupJob  // tokens expirados + sessions revocadas + audit >30 días
ReportJob   // generación async de reportes pesados
```

Reintentos: 3 automáticos con backoff. Job fallido → estado `Failed` en DB para inspección.

> Path de migración: `features = ["sqlite"]` → `["postgres"]` cuando jobs superen ~50/s.
> Los workers no cambian — solo el backend de Apalis.

---

## 8. Email — Resend + React Email — ADR 0019

```toml
resend-rs = "0.5"
```

Puerto abstracto `Mailer` — el dominio no conoce el proveedor:

```rust
// Desarrollo: imprime en log — sin credenciales necesarias
// Producción: envía via Resend (3k emails/mes gratis)
let mailer: Arc<dyn Mailer> = match config.environment {
    AppEnvironment::Development => Arc::new(LogMailer),
    _                           => Arc::new(ResendMailer::new(&config.resend_api_key)),
};
```

Plantillas en `apps/mailer/emails/` con React Email → compiladas a HTML estático.
`pnpm --filter mailer build` corre antes de compilar Rust en `just build`.

---

## 9. Almacenamiento — Tigris S3 — ADR 0020

```toml
aws-config = "1.1"
aws-sdk-s3 = "1.1"
```

API S3 estándar — si Tigris desaparece, se cambia solo `AWS_ENDPOINT_URL_S3` en `.env`.
Sin costos de egress. Las subidas desde el frontend usan Presigned URLs — los archivos
nunca pasan por el servidor Rust.

Fallback documentado: Cloudflare R2 — misma API S3, un solo cambio en `.env`.

---

## 10. Observabilidad — ADR 0016, 0015

### Fase 1 — MVP (~20MB RAM total)

```toml
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
sentry             = "0.34"
```

- Logs JSON estructurados con `request_id` en cada span
- Sentry: captura panics y errores críticos (capa gratuita)
- Axiom vía OTLP: solo trazas lentas en producción
- Healthchecks.io: ping tras Litestream backup, Apalis worker, TLS cert — ADR 0015

### Fases futuras (no en MVP)

```toml
# Fase 2
tracing-opentelemetry = "0.24"
opentelemetry-otlp    = "0.16"

# Fase 3 — solo cuando el VPS suba a $20+
# Loki + Tempo + Grafana self-hosted
```

---

## 11. Frontend — Astro SSR + Svelte 5 — ADR 0022

```
Astro SSR      → HTML desde servidor, cero JS innecesario, SEO nativo
Svelte Islands → interactividad solo donde se necesita (~5KB vs ~130KB React)
Svelte 5 Runes → estado reactivo mínimo y directo
```

| Capa | Tecnología | ADR |
|------|-----------|-----|
| Framework | Astro 5 + Svelte 5 | 0022 |
| Estilos | Tailwind v4 | 0022 |
| Componentes | shadcn-svelte + Bits UI | 0022 |
| Estado servidor | TanStack Query | 0022 |
| Validación | ArkType | 0022 |
| i18n | Paraglide JS — traducciones compiladas | 0023 |
| Tipos API | `buf generate` desde `proto/` (Fase 2) | 0027 |
| Landing page | Captura de leads + SEO | 0029 |

Los tipos TypeScript son generados por `buf generate` — **nunca escritos a mano**.

---

## 12. Deploy — Podman + Caddy + Kamal — ADR 0014

### Containerfile multi-stage — ADR 0013

```dockerfile
# Stage 1 — build
FROM rust:1.82-slim AS builder
RUN rustup target add x86_64-unknown-linux-musl
ENV SQLX_OFFLINE=true
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2 — runtime (~10MB, sin shell)
FROM gcr.io/distroless/cc-debian12
COPY --from=builder .../api /api
COPY --from=ghcr.io/benbjohnson/litestream:latest-amd64 /usr/local/bin/litestream /litestream
ENTRYPOINT ["/litestream", "replicate", "-exec", "/api"]
```

| Herramienta | Rol | ADR |
|-------------|-----|-----|
| Podman rootless | Sin daemon root — superficie de ataque mínima | 0014 |
| Caddy | TLS automático Let's Encrypt + headers de seguridad | 0014 |
| Kamal | Zero-downtime deploy + rollback en ~5 segundos | 0014 |
| VPS $5 | 1 vCPU / 1GB RAM — suficiente para el MVP | 0031 |
| Litestream sidecar | RPO ~1 segundo hacia S3/Tigris | 0004 |

```bash
just deploy    # audit + test + kamal deploy + ping healthchecks
kamal rollback # rollback en ~5 segundos si algo falla
```

---

## 13. Testing — 4 capas — ADR 0010

```toml
# dev-dependencies
tokio    = { features = ["test", "macros"] }
mockall  = "0.13"
reqwest  = { features = ["json"] }
httpmock = "0.7"
```

```bash
cargo install cargo-nextest  # 3-5x más rápido que cargo test
```

| Capa | Dónde | Velocidad | Característica |
|------|-------|-----------|----------------|
| 1 Domain | `crates/domain/src` | ~ms | Sin async, sin DB, sin mocks |
| 2 Application | `crates/application/src` | ~ms | Mocks con mockall |
| 3 Integración | `crates/database/tests` | ~s | SQLite `:memory:` real |
| 4 E2E | `apps/api/tests` | ~10s | reqwest contra servidor real |

```bash
just test      # capas 1-3 — para desarrollo local
just test-all  # capas 1-4 incluyendo E2E — para CI
```

---

## 14. Dev tools — ADR 0012

| Herramienta | Rol | ADR |
|-------------|-----|-----|
| `just` | Runner de comandos — reemplaza Makefile | 0012 |
| `pnpm` | Gestor de paquetes JS con workspaces | 0012 |
| `lefthook` | Git hooks — fmt en pre-commit, lint+test en pre-push | 0012 |
| `cargo-watch` | Hot reload Rust | 0012 |
| `cargo-nextest` | Test runner 3-5x más rápido | 0010 |
| `cargo-deny` | Auditoría de licencias y CVEs | 0011 |
| `cargo-audit` | Vulnerabilidades RustSec | 0011 |
| `sqlx-cli` | Migraciones (`just migrate`) | 0005 |
| `buf` | Genera tipos multi-plataforma desde proto (Fase 2) | 0027 |

```bash
just setup   # instala todo y deja el entorno listo en <5 minutos
just --list  # ver todos los comandos disponibles
```

---

## 15. Escalado sin reescribir — ADR 0031

### Fase 1 — MVP (hoy)

```
✅ Axum + SQLite WAL + RBAC completo         ADR 0003, 0004, 0006
✅ PASETO v4 + argon2id + Sessions           ADR 0008
✅ Litestream → S3/Tigris                    ADR 0004, 0020
✅ Apalis jobs + Resend email                ADR 0018, 0019
✅ Moka cache Decorator                      ADR 0017
✅ tower-governor rate limiting               ADR 0009
✅ Utoipa + Scalar /docs                     ADR 0021
✅ Sentry + OTLP + Healthchecks.io           ADR 0016, 0015
✅ Podman + Caddy + Kamal                    ADR 0014
✅ cargo-deny + cargo-audit en CI            ADR 0010
✅ Landing page + captura de leads           ADR 0029
```

### Fase 2 — Crecimiento

```
🟡 NATS JetStream — eventos entre módulos    ADR 0025, 0026
🟡 ConnectRPC + buf — multi-plataforma       ADR 0027
🟡 Paraglide i18n                            ADR 0023
🟡 Local-First SQLite Wasm                   ADR 0024
🟡 Sintonía CLI — desde módulo 4             ADR 0028
🟡 Tauri desktop + App móvil                 ADR 0030
```

### Fase 3 — Escala real

```
🔴 SurrealDB o PostgreSQL                    ADR F-001, F-002
🔴 Stack OTel completo (Loki + Tempo + Grafana)
🔴 KMP + UniFFI para mobile nativo           ADR 0030
```

> No añadir Fase 2 ni 3 hasta que el problema concreto exista.

---

## 16. Stack final resumido

### Backend

```toml
tokio          = { features = ["full"] }
axum           = "0.8"
tower-http     = "0.6"
tower-governor = "0.4"
sqlx           = { features = ["sqlite", "runtime-tokio", "macros", "uuid", "time"] }
argon2         = "0.5"
pasetors       = { version = "0.7", features = ["v4"] }  # PASETO, no JWT
moka           = { version = "0.12", features = ["future"] }
apalis         = { version = "0.6", features = ["sqlite", "tracing", "retry"] }
resend-rs      = "0.5"
aws-sdk-s3     = "1.1"
utoipa         = { version = "4", features = ["axum_extras", "uuid"] }
utoipa-scalar  = { version = "0.1", features = ["axum"] }
tracing        = "0.1"
sentry         = "0.34"
thiserror      = "2"
config         = "0.14"
```

### Frontend

```
Astro 5        Svelte 5       TypeScript
Tailwind v4    shadcn-svelte  Bits UI
TanStack Query ArkType        Paraglide JS
buf generate   (tipos desde proto/ — ADR 0027, Fase 2)
```

### Infraestructura

```
Podman rootless   Caddy (TLS auto)   Kamal (zero-downtime)
VPS $5            Litestream → S3    Tigris (storage)
Sentry            Healthchecks.io
```

---

## Reglas de oro

| # | Regla | ADR |
|---|-------|-----|
| 1 | `crates/domain` sin dependencias externas | 0001 |
| 2 | SQL solo en `crates/database` | 0001 |
| 3 | JWT prohibido — solo PASETO v4 Local | 0008 |
| 4 | Soft Delete — nunca `DELETE` real en users | 0006 |
| 5 | Toda acción autenticada se audita | 0006 |
| 6 | Tipos TypeScript por `buf generate` | 0027 |
| 7 | `cargo-deny` + `cargo-audit` en CI | 0010 |
| 8 | Imagen distroless — ~10MB, sin shell | 0013 |
| 9 | Fail-fast en config — si falta variable, proceso no arranca | 0002 |
| 10 | No añadir Fase 2 hasta que el problema exista | 0011 |
