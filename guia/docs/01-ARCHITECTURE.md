# Arquitectura — boilerplate

## Filosofía

Sistema de alto rendimiento basado en **arquitectura hexagonal**, priorizando:

- **Ejecutabilidad desde el día uno** — MVP funcional antes que arquitectura perfecta
- **Compilador como árbitro** — las fronteras de capa las hace cumplir Rust, no convenciones
- **Type-safety de extremo a extremo** — Rust → TypeScript sin duplicación
- **Escalado sin reescritura** — el diseño correcto ahora evita migraciones dolorosas después

---

## Estructura de capas — ADR 0001

```
boilerplate/
├── apps/          # Binarios — ensamblan crates, sin lógica propia
│   ├── api/       # Servidor Axum — ADR 0003
│   ├── web/       # Astro + Svelte 5 — ADR 0022
│   └── cli/       # Sintonía CLI — ADR 0028
│
├── crates/        # El corazón — lógica de negocio con fronteras en Cargo.toml
│   ├── domain/        # Núcleo puro — solo thiserror, uuid, time, serde
│   ├── application/   # Casos de uso — solo domain
│   ├── infrastructure/# HTTP, config — application + axum
│   ├── database/      # Repositorios SQLx — domain + sqlx — ADR 0004
│   ├── auth/          # PASETO v4 + argon2id — ADR 0008
│   ├── mailer/        # Resend adapter — ADR 0019
│   ├── storage/       # Tigris/S3 adapter — ADR 0020
│   └── events/        # NATS JetStream (Fase 2) — ADR 0025
│
├── data/migrations/   # 6 migraciones RBAC + Sessions + Audit — ADR 0006
├── infra/             # Docker, Caddy, Litestream, Kamal — ADR 0014
└── docs/adr/          # 30 ADRs activos + 2 futuros
```

---

## Por qué `crates/` y no `libs/`

Con `crates/` y `Cargo.toml` propio por crate, si alguien escribe `use sqlx` en
`crates/domain/`, el compilador falla inmediatamente porque `sqlx` no está en
`[dependencies]` de ese crate. **La arquitectura hexagonal queda garantizada por
el compilador — no por code review.**

---

## Las cuatro capas y su contrato

### 1. `crates/domain/` — Núcleo puro — ADR 0001

**Lo que puede importar:** `thiserror`, `uuid`, `time`, `serde`. Nada más.
**Lo que contiene:** entidades, value objects, ports (traits), errores de dominio.

```rust
// crates/domain/src/ports/user_repository.rs
pub trait UserRepository: Send + Sync {
    async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;
    async fn save(&self, user: &User)                   -> Result<(), DomainError>;
    async fn soft_delete(&self, id: &UserId)            -> Result<(), DomainError>;
    async fn has_permission(&self, id: &UserId, perm: &str) -> Result<bool, DomainError>;
}
```

`has_permission` hace el JOIN completo `users → user_roles → role_permissions → permissions`
transparente para el dominio — no sabe que hay cuatro tablas.

### 2. `crates/application/` — Casos de uso — ADR 0001

**Lo que puede importar:** solo `domain`.
**Lo que contiene:** casos de uso que orquestan entidades y ports.

```rust
// crates/application/src/use_cases/auth/login.rs
pub async fn execute(&self, input: LoginInput) -> Result<AuthTokens, AppError> {
    let user = self.users
        .find_active_by_email(&Email::new(&input.email)?)
        .await?
        .ok_or(DomainError::InvalidCredentials)?;

    if !verify_password(&input.password, &user.password_hash)? {
        self.audit.log(AuditEntry::failed_login(&user.id)).await?;
        return Err(DomainError::InvalidCredentials.into());
    }

    let access_token = self.paseto.generate(&user.id)?;
    let refresh      = self.tokens.create_refresh(&user.id).await?;
    let session      = Session::new(user.id.clone(), input.ip, input.user_agent);
    self.sessions.save(&session).await?;
    self.audit.log(AuditEntry::login_success(&user.id)).await?;

    Ok(AuthTokens { access_token, refresh_token: refresh.token })
}
```

### 3. `crates/database/` — Implementaciones SQLx — ADR 0004

**Lo que puede importar:** `domain` + `sqlx`.
**Única crate con SQL:** si `sqlx::query!` aparece en otro crate, la arquitectura está rota.

```rust
// crates/database/src/repositories/sqlite_user_repository.rs
async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
    sqlx::query_as!(UserRow,
        "SELECT * FROM users WHERE email = ? AND deleted_at IS NULL",
        email.as_str()
    )
    .fetch_optional(&*self.pool)
    .await
    .map_err(DomainError::from)?
    .map(User::try_from)
    .transpose()
}
```

### 4. `crates/infrastructure/` — HTTP y configuración — ADR 0003

**Lo que puede importar:** `application` + `domain` + `axum` + `config`.
**Lo que contiene:** handlers, middleware, AppState.

El handler extrae, valida, delega. Nunca contiene lógica de negocio:

```rust
// crates/infrastructure/src/http/handlers/auth.rs
pub async fn login(
    State(state): State<AppState>,
    Json(body):   Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let tokens = LoginUseCase::new(&state)
        .execute(body.into())
        .await?;
    Ok(Json(tokens.into()))
}
```

---

## El esquema de base de datos — 6 migraciones — ADR 0006

Las migraciones viven en `data/migrations/` y corren automáticamente al arrancar.
Si alguna falla, el servidor no inicia (fail-fast intencional).

```
20260305135148_create_users_table.sql
  → users (Soft Delete + trigger updated_at)
  → user_roles (N:M)

20260305135149_create_rbac.sql
  → roles, permissions, role_permissions

20260305135150_create_tokens.sql
  → tokens (verificación email, reset password)

20260305135151_create_audit_logs.sql
  → audit_logs (trazabilidad completa)

20260305135152_seed_system_data.sql
  → Admin user + roles + permisos base
  → ⚠️ Password 12345678 — cambiar antes del primer deploy

20260305135153_create_sessions.sql
  → sessions (IP, user-agent, expiry, revocación)
```

### Relaciones del esquema

```
users ──────┬── user_roles ──── roles ──── role_permissions ──── permissions
            │
            ├── tokens          (verificación, reset)
            ├── sessions        (PASETO + refresh, IP + UA)
            └── audit_logs      (trazabilidad, ON DELETE SET NULL)
```

**Decisiones de diseño clave:**
- `users.deleted_at` — Soft Delete con índice parcial en email
- `sessions` separadas de `tokens` — sesiones = login activo; tokens = flujos de email
- `audit_logs.user_id` ON DELETE SET NULL — los logs sobreviven al borrado de usuario
- RBAC N:M — un usuario puede tener múltiples roles simultáneos

---

## Flujo de una petición HTTP autenticada

```
Cliente
  ↓ HTTPS
Caddy (TLS + headers de seguridad) — ADR 0014
  ↓
tower-governor rate limiter — ADR 0009
  ↓
auth_middleware
  → extrae Bearer token (v4.local.xxx — PASETO, no JWT)
  → paseto.verify() — XChaCha20-Poly1305, sin opciones inseguras
  → inyecta UserId en Extensions
  ↓
audit_middleware
  → captura método, ruta, IP, user-agent
  → registra en audit_logs post-response (solo 2xx)
  ↓
permission_middleware
  → has_permission(user_id, "recurso:acción")
  → JOIN de 4 tablas (cacheado con Moka — ADR 0017)
  → 403 si no tiene el permiso
  ↓
Axum handler (crates/infrastructure)
  → extrae DTO, delega inmediatamente al caso de uso
  ↓
Caso de uso (crates/application)
  → orquesta entidades de dominio
  ↓
Port → implementación concreta (crates/database)
  ↓
SQLite WAL → Litestream → S3 — ADR 0004
```

---

## Flujo de autenticación completo — ADR 0008

```
POST /auth/register
  → Email::new() valida y normaliza               [domain]
  → find_active_by_email() — índice parcial        [database]
  → hash_password(argon2id, OWASP params)          [auth]
  → users.save() + audit.log()                     [database]
  → EmailJob encolado en Apalis (no bloquea HTTP)  [jobs]

POST /auth/login
  → verify_password(argon2id, tiempo constante)    [auth]
  → paseto.generate_access_token(15min)            [auth]
  → tokens.create_refresh(32 bytes opacos)         [database]
  → sessions.save(ip, user_agent, expiry)          [database]
  → audit.log(login_success)                       [database]

Rutas protegidas
  → paseto.verify() atómico — XChaCha20-Poly1305  [auth]
  → has_permission(user_id, "recurso:acción")      [database + cache]
  → 403 si no tiene el permiso                     [infrastructure]
```

---

## Manejo de errores — tres niveles — ADR 0007

```
DomainError           crates/domain — reglas de negocio violadas
    ↓ From<>
AppError              crates/domain — agrupa domain + infra errors
    ↓ IntoResponse
HTTP Response         400 / 401 / 403 / 404 / 409 / 422 / 500
```

```json
{ "error": "email_already_exists", "message": "el email ya está registrado" }
{ "error": "forbidden",            "message": "requiere permiso: users:write" }
{ "error": "internal_error",       "message": "Error interno del servidor" }
```

Los errores 500 nunca exponen detalles al cliente — se loggean con `tracing::error!`. — ADR 0016

---

## Fases de escalado — ADR 0031

### Fase 1 — MVP (hoy)

```
✅ Axum + SQLite WAL + PRAGMAs             ADR 0003, 0004
✅ RBAC completo + Sessions + Audit        ADR 0006
✅ PASETO v4 + argon2id                    ADR 0008
✅ Litestream → S3/Tigris                  ADR 0004, 0020
✅ Apalis jobs + Resend email              ADR 0018, 0019
✅ Moka cache (Decorator)                  ADR 0017
✅ tower-governor rate limiting            ADR 0009
✅ Utoipa + Scalar /docs                   ADR 0021
✅ Sentry + Axiom OTLP                     ADR 0016
✅ Podman + Caddy + Kamal                  ADR 0014
✅ Healthchecks.io                         ADR 0015
✅ Landing page + captura de leads         ADR 0029
```

### Fase 2 — Crecimiento

```
🟡 NATS JetStream — eventos                ADR 0025, 0026
🟡 ConnectRPC + buf — multi-plataforma     ADR 0027
🟡 Paraglide i18n                          ADR 0023
🟡 Local-First SQLite Wasm                 ADR 0024
🟡 Sintonía CLI — desde módulo 4           ADR 0028
🟡 Tauri Desktop + Mobile                  ADR 0030
```

### Fase 3 — Escala real

```
🔴 SurrealDB o PostgreSQL                  ADR F-001, F-002
🔴 Tauri KMP + UniFFI mobile nativo        ADR 0030
🔴 Loki + Tempo + Grafana
```

> No añadir Fase 2 ni Fase 3 hasta que el problema concreto exista.

---

## Reglas de oro

| # | Regla | Garantizada por | ADR |
|---|-------|----------------|-----|
| 1 | `crates/domain` sin infraestructura | `Cargo.toml` domain | 0001 |
| 2 | SQL solo en `crates/database` | `Cargo.toml` domain/application | 0001 |
| 3 | Handlers delegan inmediatamente | Code review | 0001 |
| 4 | JWT prohibido — solo PASETO v4 Local | `jsonwebtoken` fuera del workspace | 0008 |
| 5 | Tipos TypeScript por `buf generate` | CI verifica diff en `api.ts` | 0027 |
| 6 | `Row` struct separado de entidad | Convenio `crates/database/models/` | 0004 |
| 7 | Soft Delete — nunca `DELETE` real en users | Trigger `deleted_at` en migración | 0006 |
| 8 | Toda acción autenticada auditada | Middleware `audit.rs` automático | 0006 |
| 9 | Imagen distroless — ~10MB, sin shell | ADR 0013 | 0013 |
| 10 | Fail-fast en config | `AppConfig::load()` en main | 0002 |
