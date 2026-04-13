# Roadmap — Backend

> **Stack:** Rust 2024 · Axum 0.8 · SQLx 0.8 · SQLite WAL · PASETO v4 · Apalis · Utoipa
>
> **ADRs clave:** 0001 (Arquitectura) · 0003 (Axum) · 0004 (SQLite) · 0006 (RBAC) ·
> 0007 (Errores) · 0008 (Auth) · 0009 (Rate Limit) · 0010 (Testing)

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| I | Fundación — Dominio + DB + RBAC ✅ **COMPLETO** | 100% |
| II | API — Axum + Middleware + Errores ✅ **COMPLETO** | 90% |
| III | Seguridad — Auth + RBAC + Audit | 0% |
| IV | OpenAPI + Scalar | 0% |
| V | Async — Jobs + Cache + Email | 0% |
| VI | Observabilidad | 0% |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la robustez, seguridad y observabilidad del backend:

| Herramienta | Propósito en el Backend |
| :--- | :--- |
| **`cargo-boundary`** | **Fronteras Físicas:** Automatiza la prohibición de imports ilegales entre capas (Hexagonal). |
| **`insta`** | **Snapshot Testing:** Facilita la validación de queries SQL y lógica de dominio compleja. |
| **`miette`** | **Errores Elegantes:** Genera diagnósticos de error altamente legibles en la terminal de desarrollo. |

---

## Bloque I — Fundación (Dominio + DB + RBAC) 🔥

> **NO pasar al Bloque II sin las 6 migraciones y el dominio compilando limpio.**
> **ADR 0001, 0004, 0006**

### I.1 — Pool SQLite con PRAGMAs

> **Referencia:** ADR 0004, docs/02-STACK.md L151-170, docs/03-STRUCTURE.md L296-321

```
[x] crates/database/src/pool.rs — create_pool():
    └─ Ref: docs/03-STRUCTURE.md L305 — pool.rs con PRAGMAs
    [x] PRAGMA journal_mode = WAL         ← Ref: ADR 0004 (SQLite WAL)
    [x] PRAGMA synchronous  = NORMAL      ← Ref: ADR 0004
    [x] PRAGMA temp_store   = MEMORY       ← Ref: ADR 0004
    [x] PRAGMA mmap_size    = 30000000000  ← Ref: ADR 0004
    [x] PRAGMA foreign_keys = ON          ← Ref: ADR 0004
    [x] PRAGMA cache_size   = -64000      ← Ref: ADR 0004
    [x] max_connections = 10, min_connections = 2
    [x] acquire_timeout = 5s
    [x] log_slow_statements → LevelFilter::Warn (100ms)
[x] Verificar que el pool conecta al arrancar
    └─ Ref: ADR 0002 — fail-fast si no conecta ✅ test incluido
```

### I.2 — 6 migraciones del sistema (ADR 0006) 🔥

> **Referencia:** ADR 0006, ADR 0005, docs/02-STACK.md L172-182, docs/01-ARCHITECTURE.md L139-164

```
[x] data/migrations/20260305135148_create_users_table.sql ✅
    └─ Ref: docs/01-ARCHITECTURE.md L145-148 — users + user_roles
    [x] tabla users con Soft Delete (deleted_at)  ← Ref: ADR 0006
    [x] UNIQUE INDEX parcial en email WHERE deleted_at IS NULL  ← Ref: ADR 0006
    [x] trigger trg_users_updated_at
    [x] tabla user_roles (N:M)

[x] data/migrations/20260305135149_create_rbac.sql ✅
    └─ Ref: docs/01-ARCHITECTURE.md L149-151 — roles + permissions
    [x] tabla roles
    [x] tabla permissions (formato "recurso:acción")  ← Ref: ADR 0006
    [x] tabla role_permissions (N:M)

[x] data/migrations/20260305135150_create_tokens.sql ✅
    └─ Ref: docs/01-ARCHITECTURE.md L152-153 — tokens
    [x] tabla tokens (verificación email + reset — un solo uso)
    [x] INDEX idx_tokens_user_lookup

[x] data/migrations/20260305135151_create_audit_logs.sql ✅
    └─ Ref: docs/01-ARCHITECTURE.md L155-157 — audit_logs
    [x] tabla audit_logs
    [x] FOREIGN KEY user_id ON DELETE SET NULL  ← Ref: ADR 0006
    [x] INDEX idx_audit_resource_search
    [x] INDEX idx_audit_user_history

[x] data/migrations/20260305135152_seed_system_data.sql ✅
    └─ Ref: docs/01-ARCHITECTURE.md L158-161 — seed data
    [x] INSERT OR IGNORE usuarios admin (password: 12345678)
    [x] INSERT OR IGNORE roles Admin + User
    [x] INSERT OR IGNORE permissions (users:read, users:write, audit:read, roles:read, roles:write)
        └─ Ref: ADR 0006 — permisos base
    [x] Admin recibe TODOS los permisos via CROSS JOIN
    [x] ⚠️ Cambiar password ANTES del primer deploy  ← Ref: ADR 0002 (seguridad)

[x] data/migrations/20260305135153_create_sessions.sql ✅
    └─ Ref: docs/01-ARCHITECTURE.md L162-164 — sessions
    [x] tabla sessions (IP + UA + expiry)
    [x] INDEX idx_sessions_token
    [x] INDEX parcial idx_sessions_expiry WHERE is_revoked = FALSE
    [x] trigger trg_sessions_activity

[x] just migrate → verificar 6 "Applied" en el output:
    └─ Ref: ADR 0005, docs/02-STACK.md L458 — sqlx-cli
    Applied 20260305135148/migrate create_users_table ✅
    Applied 20260305135149/migrate create_rbac ✅
    Applied 20260305135150/migrate create_tokens ✅
    Applied 20260305135151/migrate create_audit_logs ✅
    Applied 20260305135152/migrate seed_system_data ✅
    Applied 20260305135153/migrate create_sessions ✅
[x] admin@admin.com existe en la DB ✅
[~] just prepare → genera .sqlx/ para SQLX_OFFLINE=true
    └─ Ref: ADR 0005 — offline mode para CI/build (⚠️ hacer después de I.3)
```

### I.3 — Dominio puro — crates/domain/ (ADR 0001)

> **Referencia:** ADR 0001, docs/03-STRUCTURE.md L188-236, docs/02-STACK.md L93-95

> **Regla:** crates/domain/Cargo.toml solo tiene thiserror, uuid, time, serde.
> Si sqlx o axum aparecen aquí → la arquitectura está rota.

```
[x] Cargo.toml: thiserror, uuid (v4+v7), time, serde, async-trait — NADA MÁS ✅
    └─ Ref: docs/03-STRUCTURE.md L223-236, docs/02-STACK.md L556

[x] entities/user.rs ✅
    └─ Ref: docs/03-STRUCTURE.md L198-203
    [x] struct User { id, email, password_hash, name, is_active, email_verified_at, created_at, updated_at, deleted_at }
    [x] impl User: is_active(), soft_delete(), reactivate(), verify_email()  ← Ref: ADR 0006 (Soft Delete)

[x] entities/role.rs + entities/session.rs + entities/audit_log.rs ✅
    └─ Ref: docs/03-STRUCTURE.md L201-202
[x] entities/lead.rs  (para la landing page — ADR 0029) ✅
    └─ Ref: docs/03-STRUCTURE.md L203, ADR 0029

[x] value_objects/user_id.rs     (newtype UserId(Uuid)) ✅
    └─ Ref: docs/03-STRUCTURE.md L205
[x] value_objects/email.rs       (Email — validado + normalizado a minúsculas) ✅
    └─ Ref: docs/03-STRUCTURE.md L206
[x] value_objects/password_hash.rs (PasswordHash — nunca expone el hash) ✅
    └─ Ref: docs/03-STRUCTURE.md L207
[x] value_objects/permission.rs  (Permission — formato "recurso:acción") ✅
    └─ Ref: docs/03-STRUCTURE.md L208, ADR 0006

[x] ports/user_repository.rs ✅
    └─ Ref: docs/03-STRUCTURE.md L210
    [x] find_by_id, find_active_by_email, save, soft_delete, has_permission
[x] ports/session_repository.rs  (create, find_by_token, revoke, cleanup_expired) ✅
    └─ Ref: docs/03-STRUCTURE.md L212
[x] ports/audit_repository.rs    (log — insert-only) ✅
    └─ Ref: docs/03-STRUCTURE.md L213
[x] ports/token_repository.rs    (create, use_token, cleanup_expired) ✅
    └─ Ref: docs/03-STRUCTURE.md L214
[x] ports/lead_repository.rs     (save, find_by_email) ✅
    └─ Ref: docs/03-STRUCTURE.md L215, ADR 0029
[x] ports/mailer.rs              (trait Mailer: Send + Sync) ✅
    └─ Ref: docs/03-STRUCTURE.md L216, ADR 0019
[x] ports/storage_repository.rs  (trait StorageRepository: Send + Sync) ✅
    └─ Ref: docs/03-STRUCTURE.md L217, ADR 0020

[x] errors.rs ✅
    └─ Ref: docs/02-STACK.md L93-95 (thiserror), ADR 0007
    [x] InvalidEmail, InvalidPassword, InvalidPermission, InvalidId
    [x] EmailAlreadyExists, InvalidCredentials, InvalidToken
    [x] NotFound { resource }, Forbidden { message }, MissingPermission { permission }
    [x] Database(String), Internal(String)

[ ] errors/app_error.rs          (AppError + IntoResponse → HTTP — ADR 0007)
    └─ Ref: ADR 0007, docs/01-ARCHITECTURE.md L245-262
    → FASE POSTERIOR (en crate application/infrastructure)

[x] Tests unitarios ✅
    └─ Ref: ADR 0010 — capa 1 Domain
    [x] email_valido_se_crea()
    [x] email_normalizado_a_minusculas()
    [x] email_sin_arroba_invalido()
    [x] email_con_espacios_se_trimmea()
    [x] email_domain_extrae_correctamente()
    [x] password_hash_valido_se_crea()
    [x] password_hash_vacio_falla()
    [x] password_hash_sin_argon2_falla()
    [x] password_hash_no_expone_en_display()
    [x] user_id_new_genera_uuid()
    [x] user_id_parse_valido()
    [x] user_id_parse_invalido_falla()
    [x] permission_new_valido()
    [x] permission_parse_valido()
    [x] permission_parse_invalido_falla()
    [x] permission_implies_funciona()
    [x] permission_diferente_recurso_no_implica()
    [x] soft_delete_marca_deleted_at()
    [x] is_active_retorna_false_tras_soft_delete()
    [x] verify_email_works()
    [x] email_not_verified_on_creation()

[x] cargo nextest run -p domain → 24 tests pasan ✅
    └─ Ref: ADR 0010, docs/02-STACK.md L429-443
[x] cargo check -p domain → cero errores ✅
[x] grep "sqlx" crates/domain/Cargo.toml → cero resultados ✓
    └─ Ref: ADR 0001 — violación crítica si falla
```

### I.4 — Casos de uso — crates/application/ (ADR 0001)

> **Referencia:** ADR 0001, docs/03-STRUCTURE.md L240-264, docs/01-ARCHITECTURE.md L69-95

```
[x] Cargo.toml: domain + thiserror + anyhow + tokio — sin sqlx, sin axum ✅
    └─ Ref: docs/03-STRUCTURE.md L246 — solo domain como dependencia

[x] use_cases/auth/register.rs    (email → argon2id → save → EmailJob) ✅
    └─ Ref: docs/01-ARCHITECTURE.md L223-229 — flujo de register
[x] use_cases/auth/login.rs       (verify → PASETO 15min → refresh → session → audit) ✅
    └─ Ref: docs/01-ARCHITECTURE.md L230-235 — flujo de login
[x] use_cases/auth/logout.rs      (revocar session + refresh token) ✅
[x] use_cases/auth/refresh.rs     (verify refresh → nuevo PASETO + nuevo refresh) ✅

[x] use_cases/users/get_user.rs ✅
[x] use_cases/users/list_users.rs ✅
[x] use_cases/users/update_user.rs ✅
[x] use_cases/users/soft_delete_user.rs  (UPDATE deleted_at — NUNCA DELETE) ✅
    └─ Ref: ADR 0006 — Soft Delete obligatorio

[x] use_cases/leads/capture_lead.rs ✅
    └─ Ref: ADR 0029 — Landing + Leads
    [x] deduplicación silenciosa (retorna Ok si ya existe)
    [~] encola LeadWelcomeJob sin bloquear HTTP (PENDIENTE Bloque V)

[~] Tests con mockall:
    └─ Ref: ADR 0010 — capa 2 Application, docs/02-STACK.md L424-427
    [~] Tests en use_cases.rs (legacy) — 3 pasando
    [ ] registro_con_email_nuevo_funciona() — PENDIENTE mockall
    [ ] registro_con_email_duplicado_no_llama_save() — PENDIENTE mockall
    [ ] email_invalido_no_toca_la_db() — PENDIENTE mockall
    → Se agregarán al implementar repositorios (I.5) con mockall

[x] Verificar que use cases NO importan sqlx ni axum ✅
    └─ Ref: ADR 0001 — arquitectura hexagonal
    └─ grep "sqlx\|axum" crates/application/Cargo.toml = 0 resultados
```

### I.5 — Repositorios SQLx — crates/database/ (ADR 0004, 0017)

> **Referencia:** ADR 0004, ADR 0017, docs/03-STRUCTURE.md L296-321, docs/02-STACK.md L151-170

```
[x] Cargo.toml: domain + sqlx + moka + uuid + time + tracing ✅
    └─ Ref: docs/03-STRUCTURE.md L302 — único crate con sqlx

[x] models/user_row.rs     (UserRow — mapeo exacto de columnas DB) ✅
    └─ Ref: docs/03-STRUCTURE.md L314-320 — Row structs separados
[~] models/session_row.rs — PENDIENTE (implementar cuando se necesite)
[~] models/audit_row.rs — PENDIENTE (implementar cuando se necesite)
[~] models/token_row.rs — PENDIENTE (implementar cuando se necesite)
[~] models/lead_row.rs — PENDIENTE (implementar cuando se necesite)

[x] repositories/sqlite_user_repository.rs ✅
    └─ Ref: docs/01-ARCHITECTURE.md L97-115 — ejemplo SQLx
    [x] find_by_id
    [x] find_active_by_email     (usa índice parcial idx_users_email_active)
    [x] save (UPSERT con ON CONFLICT)
    [x] soft_delete              (UPDATE deleted_at — NUNCA DELETE real)
        └─ Ref: ADR 0006
    [x] has_permission           (JOIN 4 tablas con índices)
        └─ Ref: docs/01-ARCHITECTURE.md L66-68
    [x] list (paginación)

[~] repositories/cached_user_repository.rs  (Decorator Moka — ADR 0017)
    └─ Ref: docs/02-STACK.md L253-268, docs/03-STRUCTURE.md L308
    [ ] TTL 5min, max_capacity 10_000
    [ ] cache.invalidate() en save() y soft_delete() — CRÍTICO
    → PENDIENTE Bloque III (optimización)

[~] repositories/sqlite_session_repository.rs — PENDIENTE Bloque III
[~] repositories/sqlite_audit_repository.rs — PENDIENTE Bloque III
[~] repositories/sqlite_token_repository.rs — PENDIENTE Bloque III
[~] repositories/sqlite_lead_repository.rs — PENDIENTE Bloque III

[x] Tests de integración con SQLite :memory: ✅
    └─ Ref: ADR 0010, docs/02-STACK.md L437-438 — capa 3 Integración
    [x] guardar_y_recuperar_usuario()
    [x] soft_delete_oculta_el_usuario()
    [~] has_permission_con_rol_admin() → true (necesita seed data en test)
    [~] has_permission_sin_permiso() → false (necesita seed data en test)
    [~] cache_se_invalida_tras_soft_delete() — PENDIENTE caché

[x] cargo nextest run -p database → todos pasan ✅
    └─ Ref: ADR 0010
    └─ 3 tests pasan (pool + user_repository)
```

**✅ Verificación Bloque I completo:**
```bash
just migrate                    # 6 "Applied"
cargo nextest run -p domain     # verde
cargo nextest run -p application # verde
cargo nextest run -p database   # verde
```

---

## Bloque II — API (Axum + Middleware + Errores) — ADR 0003, 0007

> **Referencia:** ADR 0003 (Axum), ADR 0007 (Errores), ADR 0009 (Rate Limit), docs/02-STACK.md L121-148

### II.1 — Setup Axum

> **Referencia:** ADR 0003, docs/03-STRUCTURE.md L384-421, docs/01-ARCHITECTURE.md L117-136

```
[x] apps/api/src/main.rs ✅
    └─ Ref: docs/03-STRUCTURE.md L392-393
    [x] load config (fail-fast — ADR 0002) ✅
        └─ Ref: docs/02-STACK.md L106-118
    [x] init telemetry ✅
    [x] create_pool() ✅
    [~] migrate automático (comentado — usar `just migrate` en Windows)
        └─ Ref: ADR 0002 — fail-fast
    [x] build_state() ✅
    [x] serve con graceful shutdown SIGTERM + SIGINT ✅

[x] apps/api/src/setup.rs — build_state() composition root ✅
    └─ Ref: docs/03-STRUCTURE.md L403-420 — ejemplo de composición
    [x] user_repo: SqliteUserRepository (sin caché por ahora) ✅
    [ ] session_repo: SqliteSessionRepository — PENDIENTE Bloque III
    [ ] audit_repo:   SqliteAuditRepository — PENDIENTE Bloque III
    [ ] token_repo:   SqliteTokenRepository — PENDIENTE Bloque III
    [ ] lead_repo:    SqliteLeadRepository — PENDIENTE Bloque III
    [ ] paseto:       PasetoService — PENDIENTE Bloque III
    [ ] mailer:       build_mailer(config) — PENDIENTE Bloque V
    [ ] storage:      TigrisRepository — PENDIENTE Bloque V

[x] apps/api/src/router.rs — router modular ✅
    └─ Ref: docs/03-STRUCTURE.md L278
    [x] GET  /health                         → health_handler ✅
    [ ] POST /auth/register                  → auth::register — PENDIENTE Bloque III
    [ ] POST /auth/login                     → auth::login — PENDIENTE Bloque III
    [ ] POST /auth/refresh                   → auth::refresh — PENDIENTE Bloque III
    [ ] POST /auth/logout                    → auth::logout — PENDIENTE Bloque III
    [x] GET  /api/v1/users                   → users::list ✅
    [x] POST /api/v1/users                   → users::create ✅ (placeholder)
    [x] GET  /api/v1/users/:id               → users::get ✅
    [x] PUT  /api/v1/users/:id               → users::update ✅
    [x] DELETE /api/v1/users/:id             → users::soft_delete ✅
        └─ Ref: ADR 0006 — Soft Delete
    [ ] GET  /api/v1/audit                   → audit::list — PENDIENTE Bloque III
    [~] POST /api/v1/leads                   → leads::capture (placeholder)
        └─ Ref: ADR 0029 — PENDIENTE LeadRepository
    [ ] GET  /docs                           → Scalar — PENDIENTE Bloque IV
    [ ] GET  /openapi.json                   → ApiDoc spec — PENDIENTE Bloque IV

[ ] GET /health verifica conexión a DB antes de responder 200:
    { "status": "ok", "database": "connected", "version": "0.1.0" }
```

### II.2 — Middleware en orden (ADR 0003, 0009)

> **Referencia:** ADR 0003, ADR 0009, docs/02-STACK.md L134-148, docs/01-ARCHITECTURE.md L184-216

```
[x] 1. request_id_middleware   → x-request-id en cada request ✅
    └─ Ref: docs/02-STACK.md L138
[x] 2. trace_middleware        → logging con method, uri, status ✅
    └─ Ref: docs/02-STACK.md L139, ADR 0016
[x] 3. CompressionLayer        → gzip + brotli ✅
    └─ Ref: docs/02-STACK.md L140
[x] 4. CorsLayer               → configurable por entorno ✅
    └─ Ref: docs/02-STACK.md L141
[x] 5. TimeoutLayer            → 30 segundos ✅
    └─ Ref: docs/02-STACK.md L142
[ ] 6. Rate limit global       → tower-governor 10 req/s, burst 30
    └─ Ref: ADR 0009 — PENDIENTE
[ ] 7. Rate limit auth         → 1 req/s, burst 5 en /auth/*
    └─ Ref: ADR 0009 — PENDIENTE Bloque III
[ ] 8. Rate limit leads        → 3 req/min en /api/v1/leads
    └─ Ref: ADR 0009, ADR 0029 — PENDIENTE Bloque III
[ ] /health excluido del rate limit — PENDIENTE
    └─ Ref: ADR 0009
```

### II.3 — Manejo de errores (ADR 0007)

> **Referencia:** ADR 0007, docs/01-ARCHITECTURE.md L245-262, docs/02-STACK.md L97-105

```
[x] DomainError en crates/domain/src/errors.rs ✅
    └─ Ref: ADR 0007, docs/02-STACK.md L93-95
[x] ApiError + IntoResponse en apps/api/src/error.rs ✅
    └─ Ref: docs/03-STRUCTURE.md L220, docs/01-ARCHITECTURE.md L248-253
    [x] 400 → InvalidEmail, InvalidPassword, Validation ✅
    [x] 401 → InvalidToken, InvalidCredentials ✅
    [x] 403 → Forbidden, MissingPermission ✅
    [x] 404 → NotFound ✅
    [x] 409 → EmailAlreadyExists ✅
    [x] 422 → Validation ✅
    [x] 500 → Database, Internal (sin detalles al cliente) ✅
        └─ Ref: ADR 0016, docs/02-STACK.md L104
[x] Formato consistente: { "error": { "code": "...", "message": "..." } } ✅
    └─ Ref: docs/02-STACK.md L100-102
```

**✅ Verificación Bloque II:**
```bash
# Compilación limpia
cargo check --package api  # → 0 errores, warnings esperados

# Próximo paso: probar servidor
just migrate
cargo run --bin api &
curl http://localhost:3000/health  # → {"status":"ok", "database":"connected"}
```

---

## Bloque III — Seguridad (Auth + RBAC + Audit) — ADR 0008, 0006

> **Referencia:** ADR 0008 (PASETO), ADR 0006 (RBAC), docs/02-STACK.md L203-237, docs/01-ARCHITECTURE.md L220-241

### III.1 — crates/auth/ — argon2id + PASETO v4

> **Referencia:** ADR 0008, docs/03-STRUCTURE.md L325-336, docs/02-STACK.md L203-226

```
[x] Cargo.toml: domain + argon2 + pasetors + secrecy — SIN jsonwebtoken ✅
    └─ Ref: docs/02-STACK.md L206 — pasetors v4, NO jsonwebtoken
    [x] secrecy = { workspace = true } ✅
    [x] rand + sha2 + hex para tokens opacos ✅

[x] password.rs ✅
    └─ Ref: docs/03-STRUCTURE.md L333
    [x] hash_password(password) → argon2id, parámetros OWASP 2024 (m=19456, t=2, p=1) ✅
        └─ Ref: docs/02-STACK.md L205
    [x] verify_password(password, hash) → comparación en tiempo constante ✅
    [x] Tests unitarios ✅

[x] paseto.rs — PasetoService ✅
    └─ Ref: docs/03-STRUCTURE.md L334
    [x] new(&secret) → panic si secret ≠ 32 bytes (fail-fast) ✅
        └─ Ref: ADR 0002
    [x] generate_access_token(user_id, 15min) → "v4.local.xxx" ✅
        └─ Ref: docs/01-ARCHITECTURE.md L232, docs/02-STACK.md L213-217
    [x] verify(token) → TokenClaims ✅
    [x] Rechaza tokens JWT ("eyJ") — ADR 0008 ✅

[x] token.rs ✅
    [x] generate_opaque_token() → 32 bytes aleatorios (refresh tokens) ✅
    [x] hash_token(raw) → SHA-256 para almacenar en DB ✅
    [x] Tests unitarios ✅

[x] grep "jsonwebtoken" . → cero resultados ✓
    └─ Ref: ADR 0008 — JWT prohibido ✅
```

### III.2 — Endpoints de autenticación

> **Referencia:** docs/01-ARCHITECTURE.md L220-241 — flujo completo de auth

```
[x] POST /auth/register ✅
    └─ Ref: docs/01-ARCHITECTURE.md L223-229
    [x] Email::new() valida + normaliza ✅
    [x] find_active_by_email() → 409 si ya existe ✅
    [x] hash_password(argon2id) ✅
    [x] save user con password_hash ✅
    [ ] encolar EmailJob:Welcome — PENDIENTE Bloque V
    [x] retorna 201 + { user_id } ✅

[x] POST /auth/login ✅
    └─ Ref: docs/01-ARCHITECTURE.md L230-235
    [x] find_active_by_email() → 401 si no existe ✅
    [x] verify_password() con argon2id ✅
    [x] generate_access_token(15min) ✅
    [x] create_refresh_token() → opaque 32 bytes ✅
    [~] create_session(ip, user_agent, expiry) — PLACEHOLDER
    [~] audit.log(login_success) — PENDIENTE Bloque III.3
    [x] retorna 200 + { access_token, refresh_token } ✅
    [x] access_token empieza con "v4.local." — verificado ✅

[x] POST /auth/refresh — ESTRUCTURA LISTA
    [~] verify refresh token hash en DB — PLACEHOLDER
    [~] REVOCAR el refresh token anterior — PLACEHOLDER
    [~] generar nuevo access_token + nuevo refresh_token — PLACEHOLDER
    [ ] retorna 200 + { access_token, refresh_token } — PENDIENTE

[x] POST /auth/logout — ESTRUCTURA LISTA
    [~] revocar session — PLACEHOLDER
    [~] revocar refresh token — PLACEHOLDER
    [x] retorna 200 ✅
```

### III.3 — Middleware de Auth + RBAC

> **Referencia:** ADR 0008, ADR 0006, docs/03-STRUCTURE.md L280-283, docs/01-ARCHITECTURE.md L184-216

```
[ ] crates/infrastructure/src/http/middleware/auth.rs
    └─ Ref: docs/03-STRUCTURE.md L281
    [ ] extrae Bearer token del header Authorization
    [ ] paseto.verify(token) → UserId en Extensions
        └─ Ref: docs/01-ARCHITECTURE.md L194-196
    [ ] 401 si no hay token o es inválido
    [ ] tracing::Span::current().record("user_id", ...)
        └─ Ref: ADR 0016

[ ] crates/infrastructure/src/http/middleware/permission.rs
    └─ Ref: docs/03-STRUCTURE.md L282
    [ ] require_permission("recurso:acción") → función reutilizable
        └─ Ref: docs/02-STACK.md L227-233
    [ ] llama has_permission(user_id, perm) (cacheado con Moka)
        └─ Ref: docs/01-ARCHITECTURE.md L203-206, ADR 0017
    [ ] 403 con { "error": "forbidden", "message": "requiere permiso: x:y" }

[ ] crates/infrastructure/src/http/middleware/audit.rs
    └─ Ref: docs/03-STRUCTURE.md L283, ADR 0006
    [ ] captura method, uri, ip, user_agent
        └─ Ref: docs/01-ARCHITECTURE.md L198-200
    [ ] registra en audit_logs SOLO si response es 2xx
    [ ] fire-and-forget — no bloquea la respuesta

[ ] Verificar protección de rutas:
    [ ] GET /api/v1/users sin token → 401
    [ ] GET /api/v1/users con token sin permiso → 403
    [ ] GET /api/v1/users con token + permiso → 200
```

### III.4 — Tests E2E de seguridad (ADR 0010)

> **Referencia:** ADR 0010, docs/02-STACK.md L429-443 — testing 4 capas

```
[ ] apps/api/tests/auth_flow_test.rs
    └─ Ref: docs/02-STACK.md L438 — capa 4 E2E
    [ ] flujo_completo_register_login_acceso_logout()
    [ ] access_token_empieza_con_v4_local()   (verifica que NO es JWT)
        └─ Ref: ADR 0008
    [ ] token_expirado_retorna_401()
    [ ] refresh_token_revocado_retorna_401()
    [ ] sin_permiso_retorna_403()
    [ ] con_permiso_correcto_retorna_200()

[ ] cargo nextest run --all-targets → todos pasan
    └─ Ref: ADR 0010, docs/02-STACK.md L442-443
```

**✅ Verificación Bloque III:**
```bash
# Flujo completo desde terminal
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@admin.com","password":"12345678"}' | jq -r '.access_token')

echo $TOKEN | cut -c1-10  # debe mostrar "v4.local.."

curl http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer $TOKEN"   # → 200 con lista
```

---

## Bloque IV — OpenAPI + Scalar (ADR 0021)

> **Referencia:** ADR 0021, docs/02-STACK.md L239-250, docs/03-STRUCTURE.md L291

```
[ ] #[derive(ToSchema)] en todos los DTOs de request/response
    └─ Ref: docs/02-STACK.md L246
[ ] #[utoipa::path] en cada handler con:
    └─ Ref: docs/02-STACK.md L246
    [ ] request_body
    [ ] responses (201, 400, 401, 403, 409, 422, 500)
    [ ] security(("bearer_auth" = []))
        └─ Ref: ADR 0008 — PASETO
    [ ] tag

[ ] apps/api/src/docs.rs — ApiDoc central:
    └─ Ref: docs/03-STRUCTURE.md L291
    [ ] paths: todos los handlers registrados
    [ ] components: todos los schemas
    [ ] SecurityAddon: bearer_format = "PASETO" (no "JWT")
        └─ Ref: ADR 0008, docs/02-STACK.md L248

[ ] /docs → Scalar UI (solo en ENVIRONMENT != "production")
    └─ Ref: docs/02-STACK.md L247-248
[ ] /openapi.json → disponible en todos los entornos
    └─ Ref: docs/02-STACK.md L248

[ ] Verificar: /docs carga en browser
[ ] Verificar: /openapi.json | jq '.info.title' → "Boilerplate API"
[ ] Verificar: el esquema de seguridad muestra "PASETO" como bearer_format
    └─ Ref: ADR 0008, docs/02-STACK.md L248
```

**✅ Verificación Bloque IV:**
```bash
curl http://localhost:8080/openapi.json | jq '.info.title'
# → "Boilerplate API"
```

---

## Bloque V — Async (Jobs + Cache + Email) — ADR 0017, 0018, 0019

> **Referencia:** ADR 0017 (Cache), ADR 0018 (Jobs), ADR 0019 (Email), docs/02-STACK.md L253-318

### V.1 — Cache Moka Decorator (ADR 0017)

> **Referencia:** ADR 0017, docs/02-STACK.md L253-268, docs/01-ARCHITECTURE.md L276, docs/03-STRUCTURE.md L308

```
[ ] crates/database/src/repositories/cached_user_repository.rs
    └─ Ref: docs/03-STRUCTURE.md L308, docs/02-STACK.md L261-265
    [ ] TTL 5min, max_capacity 10_000, TTI 1min
        └─ Ref: docs/02-STACK.md L259
    [ ] find_active_by_email(): log "L1 HIT / MISS"
    [ ] cache.invalidate() en save() — CRÍTICO
        └─ Ref: docs/02-STACK.md L266 — sin esto hay datos obsoletos
    [ ] cache.invalidate() en soft_delete() — CRÍTICO
    [ ] Límite 100MB RAM total para todos los cachés
        └─ Ref: docs/02-STACK.md L267

[ ] Test obligatorio:
    [ ] cache_se_invalida_tras_soft_delete() → falla si se olvida invalidar
        └─ Ref: docs/02-STACK.md L266
```

### V.2 — Jobs con Apalis (ADR 0018)

> **Referencia:** ADR 0018, docs/02-STACK.md L274-295, docs/03-STRUCTURE.md L395-398

```
[ ] apps/api/src/jobs/email_job.rs
    └─ Ref: docs/03-STRUCTURE.md L397
    [ ] EmailJob { to, subject, template, context }
    [ ] EmailTemplate: Welcome, PasswordReset, LeadWelcome, Notification
        └─ Ref: ADR 0019
    [ ] handle_email_job() → render_template() + mailer.send()

[ ] apps/api/src/jobs/cleanup_job.rs
    └─ Ref: docs/03-STRUCTURE.md L398
    [ ] DELETE FROM tokens WHERE expires_at < datetime('now')
    [ ] DELETE FROM sessions WHERE expires_at < datetime('now') AND is_revoked = TRUE
    [ ] DELETE FROM audit_logs WHERE created_at < datetime('now', '-30 days')

[ ] apps/api/src/jobs/worker.rs — start_workers():
    └─ Ref: docs/03-STRUCTURE.md L396
    [ ] storage.setup().await → crea tablas de jobs
    [ ] 2 workers para EmailJob
    [ ] 1 worker para CleanupJob
    [ ] 1 worker para ReportJob
    [ ] RetryLayer::new(RetryPolicy::retries(3))
        └─ Ref: docs/02-STACK.md L291 — 3 reintentos con backoff
    [ ] TraceLayer en cada worker
        └─ Ref: ADR 0016

[ ] Verificar que EmailJob se encola en RegisterUseCase sin bloquear HTTP
    └─ Ref: docs/01-ARCHITECTURE.md L228
[ ] Verificar reintentos automáticos en caso de fallo
    └─ Ref: docs/02-STACK.md L291
[ ] Job fallido tras 3 intentos → estado "Failed" en DB para inspección
    └─ Ref: docs/02-STACK.md L292
```

### V.3 — Email con Resend + LogMailer (ADR 0019)

> **Referencia:** ADR 0019, docs/02-STACK.md L298-318, docs/03-STRUCTURE.md L340-350

```
[ ] crates/mailer/src/log_mailer.rs
    └─ Ref: docs/03-STRUCTURE.md L348
    [ ] impl Mailer: imprime en tracing::info! — no envía
    [ ] usar en ENVIRONMENT = "development"
        └─ Ref: docs/02-STACK.md L307-312

[ ] crates/mailer/src/resend_mailer.rs
    └─ Ref: docs/03-STRUCTURE.md L349
    [ ] impl Mailer con resend-rs
        └─ Ref: docs/02-STACK.md L301
    [ ] usar en ENVIRONMENT = "production" | "staging"
        └─ Ref: docs/02-STACK.md L310-312

[ ] apps/mailer/emails/welcome.tsx         (React Email)
    └─ Ref: docs/02-STACK.md L315, ADR 0019
[ ] apps/mailer/emails/password_reset.tsx
[ ] apps/mailer/emails/lead_welcome.tsx
    └─ Ref: ADR 0029
[ ] apps/mailer/emails/notification.tsx
[ ] pnpm --filter mailer build → compila a dist/*.html
    └─ Ref: docs/02-STACK.md L316
[ ] render_template() usa include_str!("../../../apps/mailer/dist/welcome.html")
    └─ Ref: docs/02-STACK.md L317
[ ] just build incluye pnpm --filter mailer build como primer paso
    └─ Ref: docs/02-STACK.md L316

[ ] Verificar en desarrollo: LogMailer imprime HTML en los logs
    └─ Ref: docs/02-STACK.md L311, L526
[ ] Verificar en producción: email llega al inbox
```

**✅ Verificación Bloque V:**
```bash
# Registrar usuario y verificar que el job se encola
curl -X POST http://localhost:8080/auth/register \
  -d '{"email":"nuevo@test.com","password":"password123"}'
# → Logs muestran "📧 [LogMailer] Email que se enviaría en producción"
# → Logs muestran "L1 MISS" en primera lectura, "L1 HIT" en segunda
```

---

## Bloque VI — Observabilidad (ADR 0016, 0015)

> **Referencia:** ADR 0016 (Observabilidad), ADR 0015 (Monitoreo), docs/02-STACK.md L335-358

```
[ ] apps/api/src/telemetry.rs — init_telemetry():
    └─ Ref: docs/03-STRUCTURE.md L394
    [ ] tracing JSON subscriber con EnvFilter
        └─ Ref: docs/02-STACK.md L339-342
    [ ] RUST_LOG=debug,sqlx=warn en dev / info en prod
    [ ] request_id en cada span (SetRequestIdLayer ya lo configura)
        └─ Ref: docs/02-STACK.md L138-139

[ ] Sentry SDK:
    └─ Ref: docs/02-STACK.md L342, ADR 0016
    [ ] sentry::init() con SENTRY_DSN (opcional en dev)
    [ ] tracing_sentry::layer() en el subscriber
    [ ] Verificar que un panic llega al dashboard de Sentry

[ ] OTLP hacia Axiom (solo en producción):
    └─ Ref: docs/02-STACK.md L350-358, ADR 0016
    [ ] opentelemetry_otlp::new_exporter().http()
    [ ] solo si ENVIRONMENT == "production"

[ ] Healthchecks.io (ADR 0015):
    └─ Ref: docs/02-STACK.md L348, ADR 0015
    [ ] HC_LITESTREAM_UUID en .env
    [ ] HC_DEPLOY_UUID en .env
    [ ] ping en just deploy como último paso

[ ] Verificar: logs son JSON válido
    └─ Ref: docs/02-STACK.md L345
    just dev-api 2>&1 | head -5 | jq .
```

---

## ADRs de referencia por bloque

| Bloque | ADR |
|--------|-----|
| I — Fundación | 0001, 0004, 0005, 0006, 0017 |
| II — API | 0003, 0007, 0009 |
| III — Seguridad | 0008, 0006 |
| IV — OpenAPI | 0021 |
| V — Async | 0017, 0018, 0019 |
| VI — Observabilidad | 0016, 0015 |

**Siguiente cuando el backend está completo:** → `ROADMAP-FRONTEND.md` (puede ir en paralelo desde Bloque II)

---

## Diagrama de Flujo de Bloques

```
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE I — Fundación                                                  │
│  ├─ Pool SQLite (WAL + PRAGMAs)                                       │
│  ├─ 6 Migraciones (users, roles, tokens, audit, sessions)             │
│  ├─ crates/domain (entities, value_objects, ports, errors)          │
│  ├─ crates/application (use_cases)                                   │
│  └─ crates/database (repositories SQLx + Moka cache)                   │
│     └─ Ref: ADR 0001, 0004, 0006, 0017                               │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE II — API + Middleware                                          │
│  ├─ Axum setup (main.rs, router, state composition)                  │
│  ├─ Middleware ordenado (trace, cors, rate limit, timeout)            │
│  └─ Error handling (DomainError → AppError → IntoResponse)            │
│     └─ Ref: ADR 0003, 0007, 0009                                       │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE III — Seguridad                                                │
│  ├─ crates/auth (argon2id + PASETO v4) — NO JWT                       │
│  ├─ Endpoints /auth/* (register, login, refresh, logout)            │
│  ├─ Middleware auth (verify PASETO → UserId)                          │
│  ├─ Middleware RBAC (has_permission con cache Moka)                   │
│  └─ Middleware audit (logs solo 2xx)                                  │
│     └─ Ref: ADR 0008, 0006                                            │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE IV — OpenAPI + Scalar                                          │
│  ├─ utoipa ToSchema en todos los DTOs                                │
│  ├─ #[utoipa::path] en cada handler                                  │
│  ├─ /docs → Scalar UI (solo dev/staging)                             │
│  └─ /openapi.json → spec completa                                     │
│     └─ Ref: ADR 0021                                                  │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE V — Async (Jobs + Cache + Email)                               │
│  ├─ Moka cache decorator (TTL 5min, invalidación en writes)           │
│  ├─ Apalis jobs (EmailJob, CleanupJob) con reintentos                 │
│  └─ Mailer dual (LogMailer en dev, Resend en prod)                    │
│     └─ Ref: ADR 0017, 0018, 0019                                      │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  BLOQUE VI — Observabilidad                                            │
│  ├─ Tracing JSON subscriber + request_id                              │
│  ├─ Sentry SDK (panics + errors)                                      │
│  ├─ OTLP → Axiom (solo producción)                                    │
│  └─ Healthchecks.io ping en deploy                                    │
│     └─ Ref: ADR 0016, 0015                                            │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta/Crate | URL | Útil para |
|-------------------|-----|-----------|
| **Axum** | https://docs.rs/axum/latest | Router, handlers, middleware, extractors |
| **SQLx** | https://docs.rs/sqlx/latest | Queries compile-time checked, migrations |
| **PASETO** | https://paseto.io | Tokens v4.local (no JWT), formato de claims |
| **utoipa** | https://docs.rs/utoipa/latest | OpenAPI generation, Scalar UI |
| **tower-governor** | https://docs.rs/tower-governor/latest | Rate limiting |
| **Moka** | https://docs.rs/moka/latest | In-memory cache, TTL, invalidación |
| **Apalis** | https://docs.rs/apalis/latest | Background jobs, workers, retries |
| **Argon2** | https://docs.rs/argon2/latest | Password hashing (OWASP params) |
| **pasetors** | https://docs.rs/pasetors/latest | PASETO v4 implementation |
| **thiserror** | https://docs.rs/thiserror/latest | Domain error definitions |
| **Sentry Rust** | https://docs.rs/sentry/latest | Error tracking, panic reporting |
| **Tracing** | https://docs.rs/tracing/latest | Structured logging, spans, JSON output |

---

## Troubleshooting — Backend por Bloque

### Bloque I — Fundación

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `cargo check -p domain` falla | sqlx importado en domain | Revisar `Cargo.toml` NO tenga sqlx — Ref: ADR 0001 |
| `just migrate` error | DATABASE_URL no seteada | `export DATABASE_URL=sqlite://./data/dev.db` |
| `just migrate` solo aplica 5/6 | Timestamp duplicado o orden incorrecto | Renombrar migración con timestamp posterior |
| Test de dominio lento (>100ms) | Lógica con I/O bloqueante | Domain debe ser 100% CPU, sin async |
| `cargo nextest run -p database` falla | Pool no conecta | Verificar SQLite file existe, permisos OK |

### Bloque II — API

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `cargo run --bin api` panic en arranque | Config inválida o DB no accesible | Revisar `.env.local`, fall-fast ADR 0002 |
| `/health` retorna 500 | Pool no conecta | Verificar `create_pool()` en main.rs |
| Rate limit no funciona | Middleware en orden incorrecto | `TimeoutLayer` debe ir ANTES de `GovernorLayer` |
| CORS bloquea requests del frontend | Origen no permitido | Revisar `CorsLayer` config para dev |
| Error 500 expone stack trace | AppError no oculta detalles | Verificar `Internal` variant no expone info |

### Bloque III — Seguridad

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `access_token` empieza con "eyJ" | Usando JWT en lugar de PASETO | Revisar `paseto.rs` — Ref: ADR 0008 |
| Login siempre retorna 401 | Password hash mal generado | Verificar argon2id params en `password.rs` |
| Refresh token no rota | No se revoca el anterior | Revisar `refresh.rs` — Ref: ADR 0008 |
| RBAC siempre retorna 403 | Cache Moka stale | Verificar `invalidate()` en `permission.rs` |
| Audit logs vacíos | Middleware audit no registrado | Revisar orden de middleware en main.rs |

### Bloque IV — OpenAPI

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `/docs` 404 | Scalar no configurado | Revisar `utoipa-scalar` en `Cargo.toml` |
| `/openapi.json` inválido | Handler sin `#[utoipa::path]` | Añadir macro a cada handler |
| Schema muestra "JWT" en lugar de "PASETO" | SecurityAddon mal configurado | `bearer_format = "PASETO"` en docs.rs |
| Tipos no aparecen en Schema | Sin `#[derive(ToSchema)]` | Añadir a todos los DTOs |

### Bloque V — Async

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Email no se envía | Job no encolado | Revisar `enqueue` en `RegisterUseCase` |
| Cache nunca hace HIT | TTL muy corto o clave diferente | Verificar clave de cache en `find_by_email` |
| Datos obsoletos en cache | Falta `invalidate()` | Añadir en `save()` y `soft_delete()` — Ref: docs/02-STACK.md L266 |
| Job falla sin reintentos | `RetryLayer` no configurado | Verificar `start_workers()` — Ref: ADR 0018 |
| React Email no compila | Node modules desactualizados | `pnpm --filter mailer install` |

### Bloque VI — Observabilidad

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Logs no son JSON | Subscriber mal configurado | `tracing_subscriber::fmt::json()` |
| Sentry no recibe panics | `tracing_sentry` layer no añadido | Revisar `telemetry.rs` — Ref: ADR 0016 |
| request_id no aparece | `SetRequestIdLayer` no configurado | Revisar middleware orden — Ref: ADR 0003 |
| Healthchecks.io no recibe ping | `HC_DEPLOY_UUID` no seteada | Revisar `.env.local` — Ref: ADR 0015 |
| OTLP no envía trazas | Solo en `production` | Verificar `ENVIRONMENT` var — Ref: ADR 0016 |

---

**Nota:** Si un error persiste, revisar el ADR correspondiente listado en las referencias de cada bloque.
