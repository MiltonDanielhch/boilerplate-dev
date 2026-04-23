# ADR 0033 — Persistencia alternativa: MySQL / MariaDB

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado — alternativa válida cuando el cliente lo requiere |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (SQLite WAL — opción principal) · ADR 0031 (Escalamiento 5 Niveles) · ADR F-002 (PostgreSQL) · ADR 0001 (Arquitectura Hexagonal) |

---

## Contexto

El stack del proyecto usa SQLite WAL + Litestream (ADR 0004) como base de datos
principal. Esa decisión está tomada y es la correcta para el MVP en un VPS de $5.

Sin embargo, hay escenarios reales donde MySQL es la única opción:

- El cliente ya tiene infraestructura MySQL pagada y quiere usarla
- El equipo conoce MySQL y no quiere aprender SQLite
- El proveedor de hosting solo ofrece MySQL (cPanel, Plesk, hosting compartido)
- El proyecto debe integrarse con un sistema legacy que usa MySQL
- Se quiere usar herramientas BI o dashboards que se conectan mejor a MySQL

Este ADR documenta cómo usar MySQL con el stack Rust + Axum + SQLx manteniendo
la arquitectura hexagonal intacta.

---

## La buena noticia — la arquitectura lo hace fácil

Gracias a la arquitectura hexagonal del ADR 0001, **MySQL solo toca un crate:
`crates/database`**. El dominio, los casos de uso, los handlers y los tests
no cambian absolutamente nada.

```
crates/domain/      → sin cambios — no sabe qué DB existe
crates/application/ → sin cambios — no sabe qué DB existe
crates/auth/        → sin cambios
crates/mailer/      → sin cambios
crates/infrastructure/ → sin cambios — handlers delegan a use cases
crates/database/    → ÚNICO CRATE QUE CAMBIA
apps/api/           → cambia solo el Cargo.toml (feature de sqlx)
```

---

## Decisión

Usar **MySQL 8.0+ o MariaDB 10.6+** con **SQLx 0.8** como adaptador,
manteniendo exactamente la misma interfaz de repositorios que SQLite.

### 1. Cambios en Cargo.toml

```toml
# workspace root Cargo.toml — cambiar la feature de sqlx
sqlx = { version = "0.8", features = [
    "mysql",            # ← cambiar "sqlite" por "mysql"
    "runtime-tokio",
    "tls-native-tls",   # ← agregar TLS para MySQL
    "macros",
    "uuid",
    "time",
] }
```

```toml
# crates/database/Cargo.toml — sin cambios en dependencias
# (hereda sqlx del workspace)
[dependencies]
domain  = { path = "../domain" }
sqlx    = { workspace = true }
moka    = { workspace = true }
uuid    = { workspace = true }
time    = { workspace = true }
tracing = { workspace = true }
```

### 2. Pool MySQL — crates/database/src/pool.rs

```rust
// crates/database/src/pool.rs — versión MySQL
use sqlx::mysql::{MySqlPool, MySqlPoolOptions, MySqlConnectOptions};
use std::{str::FromStr, time::Duration};
use tracing::log::LevelFilter;

pub async fn create_pool(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(20)          // MySQL soporta más conexiones concurrentes que SQLite
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect_with(
            MySqlConnectOptions::from_str(database_url)?
                .log_statements(LevelFilter::Debug)
                .log_slow_statements(LevelFilter::Warn, Duration::from_millis(100)),
        )
        .await
}
```

```env
# .env.local — formato de connection string MySQL
DATABASE_URL=mysql://usuario:password@localhost:3306/boilerplate
# Con TLS:
DATABASE_URL=mysql://usuario:password@host:3306/boilerplate?ssl-mode=required
```

### 3. Diferencias SQL — qué cambia en las migraciones

MySQL tiene sintaxis diferente a SQLite en varios puntos críticos. Estas son todas
las diferencias que afectan a las 6 migraciones del proyecto:

| Concepto | SQLite | MySQL |
|----------|--------|-------|
| Placeholder queries | `?` | `?` (igual ✅) |
| Primary key texto | `TEXT PRIMARY KEY` | `VARCHAR(36) PRIMARY KEY` |
| Booleanos | `INTEGER (0/1)` | `TINYINT(1)` o `BOOLEAN` |
| Fechas con timezone | `DATETIME` (texto ISO) | `DATETIME` (sin timezone) |
| Auto-increment | `INTEGER PRIMARY KEY` | `INT AUTO_INCREMENT PRIMARY KEY` |
| Triggers | `CREATE TRIGGER` | `CREATE TRIGGER` (igual ✅) |
| Índice parcial | `WHERE deleted_at IS NULL` | ❌ No existe — usar índice normal o columna calculada |
| Soft Delete index | `CREATE INDEX ... WHERE deleted_at IS NULL` | `CREATE INDEX idx_users_email ON users(email)` |
| NOW() | `datetime('now')` | `NOW()` |
| Funciones de fecha | `datetime('now', '+1 hour')` | `DATE_ADD(NOW(), INTERVAL 1 HOUR)` |
| JSON nativo | ❌ Texto | ✅ `JSON` tipo nativo |
| UUID nativo | ❌ Texto | `VARCHAR(36)` o `BINARY(16)` |

**La diferencia más importante: los índices parciales**

SQLite permite `CREATE INDEX idx ON users(email) WHERE deleted_at IS NULL`.
MySQL NO soporta índices parciales con condición `WHERE`. Alternativas:

```sql
-- Opción A: Índice normal (funciona igual, solo menos eficiente)
CREATE INDEX idx_users_email ON users(email);

-- Opción B: Columna virtual computada (MySQL 5.7+)
ALTER TABLE users ADD COLUMN email_active VARCHAR(191)
  AS (IF(deleted_at IS NULL, email, NULL)) STORED;
CREATE UNIQUE INDEX idx_users_email_active ON users(email_active);
```

Para el MVP, la Opción A es suficiente. La Opción B es la solución de producción.

### 4. Las 6 migraciones adaptadas a MySQL

```sql
-- data/migrations/20260305135148_create_users_table.sql (MySQL)
CREATE TABLE IF NOT EXISTS users (
    id              VARCHAR(36)  PRIMARY KEY NOT NULL,
    username        VARCHAR(255),
    email           VARCHAR(191) NOT NULL,
    password_hash   TEXT         NOT NULL,
    email_verified  TINYINT(1)   NOT NULL DEFAULT 0,
    created_at      DATETIME     NOT NULL DEFAULT NOW(),
    updated_at      DATETIME     NOT NULL DEFAULT NOW(),
    deleted_at      DATETIME
);

-- Índice para búsqueda por email entre usuarios activos
-- MySQL no tiene índices parciales — usar índice normal
CREATE INDEX idx_users_email ON users(email);

-- Trigger para actualizar updated_at automáticamente
DELIMITER $$
CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
BEGIN
    SET NEW.updated_at = NOW();
END$$
DELIMITER ;

CREATE TABLE IF NOT EXISTS user_roles (
    user_id VARCHAR(36) NOT NULL,
    role_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

```sql
-- data/migrations/20260305135149_create_rbac.sql (MySQL)
CREATE TABLE IF NOT EXISTS roles (
    id          VARCHAR(36)  PRIMARY KEY NOT NULL,
    name        VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    created_at  DATETIME     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS permissions (
    id          VARCHAR(36)  PRIMARY KEY NOT NULL,
    name        VARCHAR(100) NOT NULL UNIQUE,  -- formato "recurso:accion"
    description TEXT,
    created_at  DATETIME     NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id       VARCHAR(36) NOT NULL,
    permission_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id)       REFERENCES roles(id)       ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);
```

```sql
-- data/migrations/20260305135150_create_tokens.sql (MySQL)
CREATE TABLE IF NOT EXISTS tokens (
    id          VARCHAR(36)  PRIMARY KEY NOT NULL,
    user_id     VARCHAR(36)  NOT NULL,
    token_hash  VARCHAR(64)  NOT NULL UNIQUE,
    token_type  VARCHAR(50)  NOT NULL,          -- 'email_verification' | 'password_reset'
    expires_at  DATETIME     NOT NULL,
    used_at     DATETIME,
    created_at  DATETIME     NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_tokens_user ON tokens(user_id);
CREATE INDEX idx_tokens_expires ON tokens(expires_at);
```

```sql
-- data/migrations/20260305135151_create_audit_logs.sql (MySQL)
CREATE TABLE IF NOT EXISTS audit_logs (
    id          VARCHAR(36)  PRIMARY KEY NOT NULL,
    user_id     VARCHAR(36),                    -- NULL si el usuario fue borrado
    action      VARCHAR(255) NOT NULL,
    resource    VARCHAR(255) NOT NULL,
    ip_address  VARCHAR(45),                    -- IPv4 o IPv6
    user_agent  TEXT,
    created_at  DATETIME     NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_audit_user    ON audit_logs(user_id);
CREATE INDEX idx_audit_created ON audit_logs(created_at);
CREATE INDEX idx_audit_resource ON audit_logs(resource);
```

```sql
-- data/migrations/20260305135152_seed_system_data.sql (MySQL)
-- admin@admin.com / 12345678 (argon2id hash)
-- ⚠️ CAMBIAR ANTES DEL PRIMER DEPLOY
INSERT IGNORE INTO users (id, email, password_hash, email_verified, created_at, updated_at)
VALUES (
    'usr_admin_00000000000000000001',
    'admin@admin.com',
    '$argon2id$v=19$m=65536,t=3,p=4$...',
    1,
    NOW(),
    NOW()
);

INSERT IGNORE INTO roles (id, name, description) VALUES
    ('rol_admin_00000000000000000001', 'Admin', 'Administrador del sistema'),
    ('rol_user_000000000000000000001', 'User',  'Usuario estándar');

INSERT IGNORE INTO permissions (id, name, description) VALUES
    ('prm_0000000000000000000000001', 'users:read',  'Ver usuarios'),
    ('prm_0000000000000000000000002', 'users:write', 'Crear y editar usuarios'),
    ('prm_0000000000000000000000003', 'audit:read',  'Ver auditoría'),
    ('prm_0000000000000000000000004', 'roles:read',  'Ver roles'),
    ('prm_0000000000000000000000005', 'roles:write', 'Gestionar roles');

-- Admin recibe todos los permisos
INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT 'rol_admin_00000000000000000001', id FROM permissions;

INSERT IGNORE INTO user_roles (user_id, role_id)
VALUES ('usr_admin_00000000000000000001', 'rol_admin_00000000000000000001');
```

```sql
-- data/migrations/20260305135153_create_sessions.sql (MySQL)
CREATE TABLE IF NOT EXISTS sessions (
    id          VARCHAR(36)  PRIMARY KEY NOT NULL,
    user_id     VARCHAR(36)  NOT NULL,
    token_hash  VARCHAR(64)  NOT NULL UNIQUE,
    ip_address  VARCHAR(45),
    user_agent  TEXT,
    expires_at  DATETIME     NOT NULL,
    is_revoked  TINYINT(1)   NOT NULL DEFAULT 0,
    created_at  DATETIME     NOT NULL DEFAULT NOW(),
    last_used_at DATETIME    NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_sessions_user    ON sessions(user_id);
CREATE INDEX idx_sessions_token   ON sessions(token_hash);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
```

### 5. Repositorios — qué cambia en el código Rust

Los repositorios son casi idénticos. SQLx usa `?` como placeholder en ambas DBs.
Los cambios son mínimos:

```rust
// crates/database/src/pool.rs
// ANTES (SQLite):
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
pub async fn create_pool(url: &str) -> Result<SqlitePool, sqlx::Error>

// DESPUÉS (MySQL):
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
pub async fn create_pool(url: &str) -> Result<MySqlPool, sqlx::Error>
```

```rust
// crates/database/src/repositories/mysql_user_repository.rs
use sqlx::MySqlPool;
use std::sync::Arc;
use domain::{User, UserId, Email, PasswordHash, DomainError, UserRepository};
use crate::models::UserRow;

pub struct MySqlUserRepository {
    pool: Arc<MySqlPool>,
}

impl MySqlUserRepository {
    pub fn new(pool: Arc<MySqlPool>) -> Self { Self { pool } }
}

impl UserRepository for MySqlUserRepository {
    async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        // El placeholder ? es igual que en SQLite — no cambia
        // La diferencia: MySQL sí distingue mayúsculas en LIKE pero no en =
        sqlx::query_as!(UserRow,
            "SELECT id, username, email, password_hash, email_verified,
                    created_at, updated_at, deleted_at
             FROM users
             WHERE email = ? AND deleted_at IS NULL",
            email.as_str()
        )
        .fetch_optional(&*self.pool)
        .await?
        .map(row_to_user)
        .transpose()
    }

    async fn has_permission(&self, id: &UserId, perm: &str) -> Result<bool, DomainError> {
        // El JOIN de 4 tablas es idéntico — MySQL lo ejecuta igual
        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as `count: i64`
               FROM users u
               JOIN user_roles ur       ON ur.user_id       = u.id
               JOIN role_permissions rp ON rp.role_id       = ur.role_id
               JOIN permissions p       ON p.id             = rp.permission_id
               WHERE u.id = ? AND p.name = ? AND u.deleted_at IS NULL"#,
            id.as_str(),
            perm,
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(count > 0)
    }

    async fn soft_delete(&self, id: &UserId) -> Result<(), DomainError> {
        // MySQL usa NOW() en lugar de datetime('now') de SQLite
        let affected = sqlx::query!(
            "UPDATE users SET deleted_at = NOW() WHERE id = ? AND deleted_at IS NULL",
            id.as_str()
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();

        if affected == 0 { return Err(DomainError::NotFound { resource: "user".into() }); }
        Ok(())
    }
}
```

**Diferencia en el tipo de fecha:** MySQL devuelve `chrono::NaiveDateTime` en lugar
del `String` ISO que devuelve SQLite. Hay que ajustar el UserRow:

```rust
// crates/database/src/models/user_row.rs — versión MySQL
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id:             String,
    pub username:       Option<String>,
    pub email:          String,
    pub password_hash:  String,
    pub email_verified: bool,
    // MySQL devuelve NaiveDateTime (sin timezone) — convertir a OffsetDateTime en row_to_user()
    pub created_at:     chrono::NaiveDateTime,
    pub updated_at:     chrono::NaiveDateTime,
    pub deleted_at:     Option<chrono::NaiveDateTime>,
}
```

```toml
# Agregar chrono a crates/database/Cargo.toml
chrono = { version = "0.4", features = ["serde"] }
```

### 6. SQLX_OFFLINE — preparar el cache sin DB activa

Con MySQL, el paso de preparar el cache offline es igual pero apunta a MySQL:

```bash
# Necesita MySQL corriendo localmente o via Docker
export DATABASE_URL="mysql://root:password@localhost:3306/boilerplate"
cargo sqlx migrate run
cargo sqlx prepare --workspace

# Verificar que .sqlx/ se actualizó
git add .sqlx/
git commit -m "chore: actualizar cache sqlx para MySQL"
```

### 7. Docker para desarrollo local

```yaml
# infra/docker/compose.dev.yml — MySQL para desarrollo
services:
  mysql:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: devpassword
      MYSQL_DATABASE:      boilerplate
      MYSQL_USER:          boilerplate
      MYSQL_PASSWORD:      devpassword
    ports:
      - "3306:3306"
    volumes:
      - mysql_data:/var/lib/mysql
    command: >
      --character-set-server=utf8mb4
      --collation-server=utf8mb4_unicode_ci
      --default-authentication-plugin=mysql_native_password
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost"]
      interval: 5s
      timeout: 3s
      retries: 10

volumes:
  mysql_data:
```

```bash
# Arrancar MySQL en desarrollo
docker compose -f infra/docker/compose.dev.yml up -d mysql

# Verificar que está listo
docker compose -f infra/docker/compose.dev.yml exec mysql \
  mysql -u boilerplate -pdevpassword boilerplate -e "SHOW TABLES;"

# Correr migraciones
export DATABASE_URL="mysql://boilerplate:devpassword@localhost:3306/boilerplate"
just migrate
```

### 8. En producción — MySQL con Coolify o Kamal

**Con Coolify (ADR 0032):**
MySQL se agrega como servicio en el dashboard con un clic:

```
Coolify Dashboard → New Resource → Database → MySQL 8.0
→ Configurar nombre de DB, usuario y password
→ Copiar el connection string interno al servicio de la API
```

El connection string interno en Coolify usa el hostname del contenedor:
```env
DATABASE_URL=mysql://boilerplate:password@mysql-service:3306/boilerplate
```

**Con Kamal (ADR 0014):**

```yaml
# infra/kamal/deploy.yml — agregar accessory para MySQL
accessories:
  mysql:
    image: mysql:8.0
    host: IP_DEL_VPS
    port: 3306
    env:
      clear:
        MYSQL_DATABASE: boilerplate
      secret:
        - MYSQL_ROOT_PASSWORD
        - MYSQL_PASSWORD
    volumes:
      - /data/mysql:/var/lib/mysql
```

---

## Apalis con MySQL — jobs asíncronos (ADR 0018)

Apalis también soporta MySQL. El cambio es solo en la feature:

```toml
# apps/api/Cargo.toml
apalis     = { version = "0.6", features = ["mysql", "tracing", "retry"] }
apalis-sql = { version = "0.6", features = ["mysql"] }
```

```rust
// apps/api/src/jobs/worker.rs
use apalis_sql::mysql::MysqlStorage;

let storage = MysqlStorage::setup(&pool).await?;
// El resto del código de workers no cambia
```

---

## Comparación completa: SQLite vs MySQL para este stack

| Aspecto | SQLite (ADR 0004) | MySQL |
|---------|-----------------|-------|
| **Proceso** | In-process (dentro del binario) | Proceso separado — cliente/servidor |
| **Setup** | Cero — el archivo se crea solo | Instalar MySQL, crear DB, usuario, permisos |
| **RAM** | ~0MB extra | ~300-500MB para el proceso MySQL |
| **VPS mínimo** | $5 (1GB RAM) | $10 (2GB RAM) recomendado |
| **Backup** | Litestream → S3 (automático) | mysqldump + cron, o replicación MySQL |
| **Concurrencia** | WAL: múltiples lectores, 1 escritor | Múltiples lectores Y escritores simultáneos |
| **Índices parciales** | ✅ Nativo | ❌ No soportado — workaround necesario |
| **Tipos de datos** | Flexible (duck typing) | Estricto (VARCHAR, INT, DATETIME exactos) |
| **UUID** | TEXT | VARCHAR(36) |
| **Booleanos** | INTEGER (0/1) | TINYINT(1) |
| **Fechas** | TEXT ISO 8601 | DATETIME (sin timezone) |
| **JSON** | ❌ Texto | ✅ Nativo |
| **Consultas full-text** | Limitado | FTS nativo |
| **Writes/s MVP** | ~100-1000 | >10,000 |
| **Escala horizontal** | ❌ (un solo nodo) | ✅ Replicación maestro-esclavo |
| **Cambios en el código** | — (base) | Solo `crates/database/` + migraciones |
| **Tiempo de setup** | 0 minutos | ~30 minutos para dev + prod |

---

## Cuándo elegir MySQL sobre SQLite

**Elegir MySQL si:**
- El cliente ya tiene una instancia MySQL pagada y quiere usarla
- El hosting del proyecto es cPanel/Plesk que solo ofrece MySQL
- El proyecto requiere integrarse con herramientas BI (Metabase, Grafana) que prefieren MySQL
- El equipo ya conoce MySQL y no quiere aprender SQLite
- Se necesitan >1000 escrituras por segundo desde el inicio

**Mantener SQLite (ADR 0004) si:**
- Es un MVP nuevo sin restricciones de infraestructura
- El VPS es de $5 con 1GB RAM — MySQL consume demasiado
- Se quiere simplicidad operativa máxima
- La aplicación tiene lecturas >> escrituras

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| **MariaDB** | Compatible 100% con MySQL — usar las mismas migraciones y el mismo driver SQLx. Elegir MariaDB si el hosting la ofrece en lugar de MySQL |
| **PostgreSQL** | Documentado en ADR F-002 — mejor opción que MySQL para escala horizontal. Si se va a salir de SQLite, PostgreSQL es superior a MySQL para este stack |
| **ORM (SeaORM, Diesel)** | Los ports del dominio ya abstraen la DB — un ORM añade complejidad sin beneficio en esta arquitectura |

---

## Consecuencias

### ✅ Positivas

- MySQL es compatible con SQLx 0.8 — el driver es puro Rust sin unsafe
- Solo `crates/database/` cambia — el dominio, casos de uso y handlers son intocables
- El placeholder `?` es igual que en SQLite — el 90% del SQL funciona igual
- Apalis, Moka y el resto del stack no necesitan cambios
- MariaDB es drop-in replacement — las mismas migraciones funcionan

### ⚠️ Negativas / Trade-offs

- **MySQL requiere proceso separado:** consume ~300-500MB de RAM extra, necesita VPS más grande
  → Mitigación: VPS de $10 (2GB RAM) es suficiente para MySQL + la API
- **Sin índices parciales:** la eficiencia de la búsqueda por email en usuarios activos baja ligeramente
  → Mitigación: Opción B (columna virtual computada) en producción — Opción A suficiente en MVP
- **Fechas sin timezone:** MySQL devuelve `NaiveDateTime` en lugar de `OffsetDateTime`
  → Mitigación: convertir en `row_to_user()` asumiendo UTC — documentar la convención
- **Backup más complejo:** `mysqldump` + cron o replicación — no hay equivalente tan simple como Litestream
  → Mitigación: Coolify y servicios MySQL gestionados incluyen backup automático a S3
- **El `cargo sqlx prepare` necesita MySQL corriendo:** más fricción en desarrollo sin Docker
  → Mitigación: `docker compose up -d mysql` antes de compilar — el `just setup` lo puede incluir

### Decisiones derivadas

- `crates/database/src/pool.rs` tiene la única referencia a `MySqlPool` — el resto del código es genérico
- Los modelos Row necesitan `chrono::NaiveDateTime` en lugar de `String` para las fechas
- El `SQLX_OFFLINE=true` en el Containerfile sigue funcionando — solo regenerar el cache `.sqlx/`
- Si el equipo tiene dudas entre MySQL y PostgreSQL, elegir PostgreSQL (ADR F-002) — tiene mejor soporte de tipos, índices parciales nativos y sintaxis más estándar
