# ADR 0006 — Esquema Base: RBAC + Sessions + Auditoría

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (SQLite), ADR 0008 (Auth), ADR 0005 (Migraciones), ADR 0007 (Errores) |

---

## Contexto

El ADR 0005 define cómo evolucionan las migraciones pero no decide el esquema inicial.
Arrancar sin RBAC, auditoría y sesiones correctamente diseñados desde el inicio genera
deuda técnica difícil de saldar una vez que hay datos en producción.

Necesitamos desde el día uno:

- **Control de acceso granular** — no solo "autenticado sí/no" sino "¿tiene permiso X?"
- **Soft Delete en usuarios** — los datos nunca se borran, los emails pueden reutilizarse
- **Trazabilidad completa** — saber quién hizo qué, cuándo y desde dónde
- **Sesiones con contexto** — IP y user-agent para detectar accesos sospechosos

---

## Decisión

6 migraciones que establecen la infraestructura completa de autenticación y autorización
desde el arranque del proyecto.

---

## Migración 1 — Usuarios con Soft Delete

```sql
-- data/migrations/20260305135148_create_users_table.sql
CREATE TABLE IF NOT EXISTS users (
    id             TEXT     PRIMARY KEY NOT NULL,
    username       TEXT,
    email          TEXT     NOT NULL,
    password_hash  TEXT     NOT NULL,
    avatar_url     TEXT,
    email_verified BOOLEAN  NOT NULL DEFAULT FALSE,
    created_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at     DATETIME           -- Soft Delete — NULL = activo
);

-- Índice parcial: un email solo es único entre usuarios activos
-- Permite reutilizar el email de una cuenta borrada
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_active
ON users(email) WHERE deleted_at IS NULL;

-- Trigger: updated_at se actualiza automáticamente
CREATE TRIGGER IF NOT EXISTS trg_users_updated_at
AFTER UPDATE ON users FOR EACH ROW
BEGIN
    UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;
```

## Migración 2 — RBAC N:M

```sql
-- data/migrations/20260305135149_create_rbac.sql
CREATE TABLE IF NOT EXISTS roles (
    id          TEXT     PRIMARY KEY NOT NULL,
    name        TEXT     NOT NULL UNIQUE,
    description TEXT,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS permissions (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL UNIQUE, -- Formato: "recurso:acción" (ej: "users:write")
    description TEXT
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id       TEXT NOT NULL,
    permission_id TEXT NOT NULL,
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id)       REFERENCES roles(id)       ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

-- Un usuario puede tener múltiples roles simultáneamente
CREATE TABLE IF NOT EXISTS user_roles (
    user_id     TEXT     NOT NULL,
    role_id     TEXT     NOT NULL,
    assigned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);
```

## Migración 3 — Tokens de un solo uso

```sql
-- data/migrations/20260305135150_create_tokens.sql
CREATE TABLE IF NOT EXISTS tokens (
    id         TEXT     PRIMARY KEY NOT NULL,
    user_id    TEXT     NOT NULL,
    token_hash TEXT     NOT NULL UNIQUE,
    type       TEXT     NOT NULL,  -- "email_verification" | "password_reset"
    expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    used       BOOLEAN  DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_tokens_user_lookup ON tokens(user_id, type);
```

## Migración 4 — Auditoría

```sql
-- data/migrations/20260305135151_create_audit_logs.sql
CREATE TABLE IF NOT EXISTS audit_logs (
    id          TEXT     PRIMARY KEY NOT NULL,
    user_id     TEXT,      -- NULL si el usuario fue borrado (ON DELETE SET NULL)
    action      TEXT     NOT NULL,   -- "POST /auth/login"
    resource    TEXT     NOT NULL,   -- "users"
    resource_id TEXT,
    payload     TEXT,                -- JSON con contexto adicional
    ip_address  TEXT,
    user_agent  TEXT,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
    -- ON DELETE SET NULL: los logs sobreviven aunque el usuario sea borrado
);

CREATE INDEX IF NOT EXISTS idx_audit_resource_search ON audit_logs(resource, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_user_history    ON audit_logs(user_id, created_at);
```

## Migración 5 — Seed del sistema

```sql
-- data/migrations/20260305135152_seed_system_data.sql
-- INSERT OR IGNORE: idempotente — puede correr múltiples veces

-- ⚠️ Password: 12345678 — cambiar ANTES del primer deploy a producción
INSERT OR IGNORE INTO users (id, username, email, password_hash, email_verified) VALUES
('user_admin_3026', 'admin', 'admin@admin.com',
 '$argon2id$v=19$m=19456,t=2,p=1$pUHJ1Wcp6e6KUSiVvEfNjA$29CF+JW7MEf13DJhfxV2MKp0Yq7xVUpL3fyu92mhv3E',
 1);

INSERT OR IGNORE INTO roles (id, name, description) VALUES
('role_00000001', 'Admin', 'Acceso total al sistema'),
('role_00000002', 'User',  'Acceso estándar de usuario');

INSERT OR IGNORE INTO permissions (id, name, description) VALUES
('perm_00000001', 'users:read',  'Ver lista de usuarios'),
('perm_00000002', 'users:write', 'Crear y editar usuarios'),
('perm_00000003', 'audit:read',  'Ver logs de auditoría'),
('perm_00000004', 'roles:read',  'Ver roles y permisos'),
('perm_00000005', 'roles:write', 'Gestionar roles y permisos');

-- Admin recibe TODOS los permisos
INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p WHERE r.name = 'Admin';

INSERT OR IGNORE INTO user_roles (user_id, role_id)
SELECT u.id, r.id FROM users u CROSS JOIN roles r
WHERE u.username = 'admin' AND r.name = 'Admin';
```

## Migración 6 — Sesiones

```sql
-- data/migrations/20260305135153_create_sessions.sql
CREATE TABLE IF NOT EXISTS sessions (
    id               TEXT     PRIMARY KEY NOT NULL,
    user_id          TEXT     NOT NULL,
    session_token    TEXT     NOT NULL UNIQUE,
    ip_address       TEXT,
    user_agent       TEXT,
    expires_at       DATETIME NOT NULL,
    created_at       DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_activity_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_revoked       BOOLEAN  DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sessions_token   ON sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expiry  ON sessions(expires_at) WHERE is_revoked = FALSE;

CREATE TRIGGER IF NOT EXISTS trg_sessions_activity
AFTER UPDATE ON sessions FOR EACH ROW
BEGIN
    UPDATE sessions SET last_activity_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;
```

---

## Mapa de relaciones

```
users
  ├── user_roles ──── roles ──── role_permissions ──── permissions
  ├── tokens          (verificación email, reset — un solo uso)
  ├── sessions        (estado de login — IP + UA + expiración)
  └── audit_logs      (trazabilidad — ON DELETE SET NULL)
```

---

## Implementación en Rust — verificación de permisos

```rust
// crates/database/src/repositories/sqlite_user_repository.rs
async fn has_permission(&self, user_id: &UserId, permission: &str) -> Result<bool, DomainError> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count: i64"
        FROM users u
        JOIN user_roles ur       ON ur.user_id       = u.id
        JOIN role_permissions rp ON rp.role_id       = ur.role_id
        JOIN permissions p       ON p.id             = rp.permission_id
        WHERE u.id        = ?
          AND p.name      = ?
          AND u.deleted_at IS NULL
        "#,
        user_id.as_str(),
        permission,
    )
    .fetch_one(&*self.pool)
    .await
    .map_err(DomainError::from)?;

    Ok(count > 0)
}
```

### Middleware de permisos en Axum

```rust
// crates/infrastructure/src/http/middleware/permission.rs
pub fn require_permission(perm: &'static str)
    -> impl Fn(State<AppState>, Extension<UserId>, Request, Next) -> impl Future
{
    move |State(state), Extension(user_id), req, next| async move {
        let allowed = state.user_repo
            .has_permission(&user_id, perm)
            .await
            .map_err(|_| AppError::Internal("permission check failed".into()))?;

        if !allowed {
            return Err(AppError::Domain(DomainError::Forbidden {
                permission: perm.to_string(),
            }));
        }

        Ok(next.run(req).await)
    }
}

// Uso en el router:
Router::new()
    .route("/api/v1/users", get(list_users)
        .layer(middleware::from_fn_with_state(state.clone(), require_permission("users:read"))))
    .route("/api/v1/users", post(create_user)
        .layer(middleware::from_fn_with_state(state.clone(), require_permission("users:write"))))
```

---

## Output esperado de `just migrate`

```
Applied 20260305135148/migrate create_users_table
Applied 20260305135149/migrate create_rbac
Applied 20260305135150/migrate create_tokens
Applied 20260305135151/migrate create_audit_logs
Applied 20260305135152/migrate seed_system_data
Applied 20260305135153/migrate create_sessions
```

---

## Decisiones de diseño

| Decisión | Razonamiento |
|----------|-------------|
| Soft Delete con índice parcial | Permite reutilizar emails de cuentas borradas |
| RBAC N:M | Un usuario puede tener Admin + Moderator simultáneamente |
| Tokens separados de sesiones | Tokens = flujos de email (un uso); Sesiones = estado de login activo |
| `audit_logs.user_id ON DELETE SET NULL` | Los logs de auditoría son evidencia — sobreviven al borrado de usuario |
| Triggers para `updated_at` y `last_activity_at` | La DB garantiza consistencia automáticamente |
| `INSERT OR IGNORE` en seeds | Idempotente — puede correr en cualquier entorno |
| Permisos en formato `"recurso:acción"` | Legibles, auditables, escalables |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para fortalecer la seguridad y el rendimiento del sistema de identidad:

| Herramienta | Propósito en el Esquema |
| :--- | :--- |
| **`uuid` (v7)** | **IDs Ordenables:** Mejora el rendimiento de índices en tablas de alto crecimiento como `audit_logs`. |
| **`tower-sessions`** | **Gestión Automática:** Middleware para Axum que maneja el ciclo de vida de sesiones en SQLite sin código manual. |
| **`zxcvbn`** | **Seguridad de Passwords:** Validación de entropía de contraseñas para prevenir credenciales débiles. |
| **`axum-login`** | **Framework de Auth:** Gestiona el ciclo de vida de autenticación y permisos de forma integrada con Axum. |
| **`cedar-policy`** | **Motor de Políticas:** El lenguaje de Amazon para definir permisos complejos (ABAC) de forma externa al código. |
| **`moka`** | **Caché de Permisos:** Evita consultas repetitivas a la DB cacheando los permisos del usuario en memoria RAM. |

---

## Consecuencias

### ✅ Positivas

- El sistema arranca con auth completo — no hay que añadir RBAC después
- Soft Delete + Audit Logs + Sesiones = trazabilidad completa desde el día uno
- `INSERT OR IGNORE` hace las migraciones idempotentes — seguros en CI y producción
- El índice parcial en email es una optimización real — O(log n) para login
- Agregar un nuevo permiso es un `INSERT` — sin migraciones de esquema

### ⚠️ Negativas / Trade-offs

- 6 migraciones en lugar de 1
  → Cada migración es atómica e independiente — facilita el debugging
  → Si una migración falla, las anteriores están aplicadas y se puede corregir solo la fallida
- El JOIN de `has_permission` atraviesa 4 tablas
  → Mitigado con índices en todas las foreign keys
  → Mitigado con Moka cache (ADR 0017) — `has_permission` se cachea tras el primer check
- El seed incluye un usuario admin con password conocido
  → **Cambiar ANTES del primer deploy a producción** — nunca llegar a producción con esta password
  → `sintonia check arch` podría detectar si el hash del seed corresponde a `12345678`

### Decisiones derivadas

- `crates/domain/src/ports/user_repository.rs` incluye `has_permission()` como método del port
- El middleware `audit_middleware` registra automáticamente todas las acciones autenticadas
- La password del admin seed se rota con `just db-admin-reset` antes del primer deploy
- Agregar nuevos permisos = nuevo `INSERT OR IGNORE` en una migración nueva
