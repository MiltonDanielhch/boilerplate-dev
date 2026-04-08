# Sintonía CLI — Referencia Técnica Completa

> El framework de desarrollo interno del proyecto boilerplate.
> No es un script — es un acelerador de desarrollo con conocimiento del sistema.
> **ADR 0028** — implementar en Fase 2, después de 3 módulos a mano.

---

## Filosofía

Un CLI serio (como `cargo`, `rails new`, `laravel new`) fue escrito **después** de que
sus autores habían creado decenas de módulos a mano. Sabían exactamente qué era repetitivo,
qué variaba y qué podía romperse. El CLI capturó ese conocimiento destilado.

**Sintonía CLI no es un generador genérico.** Conoce:
- La estructura de crates del proyecto — ADR 0001
- El sistema RBAC con roles y permisos — ADR 0006
- Que los users tienen Soft Delete — nunca DELETE real — ADR 0006
- Que todo handler necesita `#[utoipa::path]` — ADR 0021
- Que todo módulo necesita sus permisos en el seed

---

## Arquitectura interna

```
apps/cli/
├── Cargo.toml           # clap + tera + walkdir
└── src/
    ├── main.rs
    ├── commands/
    │   ├── new.rs           # sintonia new <proyecto>
    │   ├── generate/
    │   │   ├── module.rs    # sintonia g module <nombre>
    │   │   ├── entity.rs    # sintonia g entity <nombre>
    │   │   └── migration.rs # sintonia g migration <nombre>
    │   ├── db.rs            # sintonia db [migrate|seed|reset|status]
    │   ├── check.rs         # sintonia check arch
    │   └── doctor.rs        # sintonia doctor
    ├── generators/
    │   ├── module.rs        # Orquesta todos los sub-generadores
    │   ├── entity.rs        # Genera entidad de dominio con Soft Delete
    │   ├── port.rs          # Genera trait del repositorio
    │   ├── repository.rs    # Genera SqliteXxxRepository + CachedXxxRepository
    │   ├── use_case.rs      # Genera casos de uso (create, get, list, soft_delete)
    │   ├── handler.rs       # Genera handler Axum con utoipa — ADR 0021
    │   ├── dto.rs           # Genera DTOs de request/response
    │   ├── migration.rs     # Genera migración SQL con Soft Delete + trigger
    │   ├── test.rs          # Genera tests unitarios e integración — ADR 0010
    │   └── rbac.rs          # Agrega permisos al seed — ADR 0006
    ├── templates/           # Plantillas Tera
    │   ├── entity.rs.tera
    │   ├── port.rs.tera
    │   ├── sqlite_repository.rs.tera
    │   ├── cached_repository.rs.tera
    │   ├── use_case_create.rs.tera
    │   ├── use_case_get.rs.tera
    │   ├── use_case_list.rs.tera
    │   ├── handler.rs.tera
    │   ├── dto.rs.tera
    │   ├── migration.sql.tera
    │   ├── test_domain.rs.tera
    │   └── test_integration.rs.tera
    └── utils/
        ├── naming.rs        # user → User → users → user_id → UserDto
        ├── fs.rs            # Idempotencia — no sobrescribir trabajo manual
        └── ast_editor.rs    # Modificar archivos con marcadores // sintonia:routes
```

---

## Comandos completos

### Proyecto

```bash
sintonia new <proyecto>
# Crea el monorepo completo:
# - workspace Cargo.toml con todos los crates
# - .env.example con todas las variables — ADR 0002
# - justfile completo — ADR 0012
# - lefthook.yml
# - las 6 migraciones base (RBAC + Sessions + Audit) — ADR 0006
# - git init + primer commit

sintonia doctor
# Verifica el entorno:
# - Rust 1.82+ instalado
# - cargo-nextest, sqlx-cli, lefthook, just, buf disponibles
# - DATABASE_URL configurada — ADR 0002
# - PASETO_SECRET tiene 32 bytes — ADR 0008
# - jsonwebtoken NO está en ningún Cargo.toml — ADR 0008
```

### Generación

```bash
sintonia g module <nombre>           # Módulo completo (ver desglose abajo)
sintonia g module <nombre> --no-rbac # Sin permisos — entidades internas del sistema
sintonia g module <nombre> --dry-run # Muestra qué crearía sin tocar nada
sintonia g entity <nombre>           # Solo la entidad de dominio + value objects básicos
sintonia g migration <nombre>        # Solo el archivo SQL con Soft Delete + trigger
```

### Base de datos

```bash
sintonia db migrate        # sqlx migrate run — ADR 0005
sintonia db seed           # cargo run --bin cli -- db seed
sintonia db reset          # sqlx database reset + migrate
sintonia db status         # sqlx migrate info
sintonia db new <nombre>   # sqlx migrate add <nombre>
```

### Verificación

```bash
sintonia check arch
# Detecta violaciones de las reglas de oro:
# - crates/domain importando sqlx o axum — Regla 1 — ADR 0001
# - crates/application importando sqlx — Regla 2 — ADR 0001
# - jsonwebtoken en cualquier Cargo.toml — Regla 3 — ADR 0008
# - DELETE real en la tabla users — Regla 4 — ADR 0006
# - Tipos TypeScript escritos a mano en api.ts — Regla 6 — ADR 0027
```

---

## Lo que genera `sintonia g module acta`

Todos los archivos, en orden:

**1. Entidad de dominio** — `crates/domain/src/entities/acta.rs`

```rust
// Generado por: sintonia g module acta
#[derive(Debug, Clone)]
pub struct Acta {
    pub id:         ActaId,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>, // Soft Delete — ADR 0006
}

impl Acta {
    pub fn new() -> Self { /* ... */ }
    pub fn is_active(&self) -> bool { self.deleted_at.is_none() }
    pub fn soft_delete(&mut self) { self.deleted_at = Some(OffsetDateTime::now_utc()); }
}
```

**2. Port** — `crates/domain/src/ports/acta_repository.rs`

```rust
pub trait ActaRepository: Send + Sync {
    async fn find_by_id(&self, id: &ActaId)       -> Result<Option<Acta>, DomainError>;
    async fn save(&self, acta: &Acta)             -> Result<(), DomainError>;
    async fn soft_delete(&self, id: &ActaId)      -> Result<(), DomainError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Acta>, DomainError>;
    async fn count(&self)                         -> Result<i64, DomainError>;
}
```

**3. Repositorio SQLite** — `crates/database/src/repositories/sqlite_acta_repository.rs`

Con `query_as!` chequeado en compile-time. Incluye `soft_delete()` con `UPDATE deleted_at`.

**4. Repositorio con caché** — `crates/database/src/repositories/cached_acta_repository.rs`

Decorator Moka con TTL 5min e invalidación en `save()` y `soft_delete()`. — ADR 0017

**5. Casos de uso** — `crates/application/src/use_cases/`

```
create_acta.rs
get_acta.rs
list_actas.rs
soft_delete_acta.rs
```

**6. Handler con Utoipa** — `crates/infrastructure/src/http/handlers/acta_handler.rs`

```rust
// Rutas correctas según la estructura del proyecto — ADR 0021
#[utoipa::path(get, path = "/api/v1/actas", ...)]
pub async fn list_actas(...) { /* delega al caso de uso */ }

#[utoipa::path(post, path = "/api/v1/actas", ...)]
pub async fn create_acta(...) { /* delega al caso de uso */ }
```

**7. DTOs** — `crates/infrastructure/src/http/handlers/acta_dto.rs`

```rust
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ActaDto { pub id: String, pub created_at: String }

#[derive(Deserialize, ToSchema)]
pub struct CreateActaRequest { /* campos del módulo */ }
```

**8. Migración** — `data/migrations/{timestamp}_create_actas.sql`

```sql
CREATE TABLE IF NOT EXISTS actas (
    id         TEXT     PRIMARY KEY NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME  -- Soft Delete
);

CREATE INDEX IF NOT EXISTS idx_actas_active ON actas(id) WHERE deleted_at IS NULL;

CREATE TRIGGER IF NOT EXISTS trg_actas_updated_at
AFTER UPDATE ON actas FOR EACH ROW
BEGIN
    UPDATE actas SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;
```

**9. Permisos RBAC** — agregados al seed — ADR 0006

```sql
INSERT OR IGNORE INTO permissions (id, name, description) VALUES
('perm_acta_001', 'actas:read',  'Ver lista de actas'),
('perm_acta_002', 'actas:write', 'Crear y editar actas');

INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.name = 'Admin' AND p.name LIKE 'actas:%';
```

**10. Tests** — generados automáticamente — ADR 0010

```rust
// Test unitario de dominio — sin deps, sin async
#[test]
fn acta_nueva_esta_activa() {
    let acta = Acta::new();
    assert!(acta.is_active());
}

#[test]
fn soft_delete_marca_deleted_at() {
    let mut acta = Acta::new();
    acta.soft_delete();
    assert!(!acta.is_active());
    assert!(acta.deleted_at.is_some());
}

// Test de integración con SQLite :memory:
#[tokio::test]
async fn guardar_y_recuperar_acta() { /* ... */ }
```

**11. Registros automáticos** en archivos existentes

```
crates/infrastructure/src/http/router.rs  ← agrega ruta /api/v1/actas
crates/domain/src/lib.rs                  ← agrega pub mod acta
apps/api/src/docs.rs                      ← registra ActaDto en ApiDoc — ADR 0021
apps/api/src/setup.rs                     ← agrega acta_repo al AppState
```

---

## Naming inteligente

```
entrada: acta

struct:    Acta
id:        ActaId
tabla:     actas
ruta:      /api/v1/actas
módulo:    acta
trait:     ActaRepository
repo:      SqliteActaRepository
cached:    CachedActaRepository
dto:       ActaDto, CreateActaRequest
handler:   acta_handler
migración: {timestamp}_create_actas
permisos:  actas:read, actas:write
```

---

## Idempotencia — nunca pierde trabajo manual

```bash
sintonia g module acta  # segunda ejecución

✔ crates/domain/src/entities/acta.rs — ya existe, skip
✔ crates/domain/src/ports/acta_repository.rs — ya existe, skip
→ crates/database/src/repositories/sqlite_acta_repository.rs — creado
→ data/migrations/20260326_create_actas.sql — creado
→ Permisos RBAC agregados al seed
```

---

## AST editing con marcadores — inamovibles

```rust
// crates/infrastructure/src/http/router.rs
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(user_router())
        .merge(project_router())
        // sintonia:routes   ← NUNCA eliminar este comentario
}

// apps/api/src/docs.rs
#[derive(OpenApi)]
#[openapi(
    paths(user_handler::list_users, user_handler::create_user),
    // sintonia:paths   ← NUNCA eliminar
    components(schemas(UserDto, CreateUserRequest)),
    // sintonia:schemas ← NUNCA eliminar
)]
pub struct ApiDoc;
```

Resultado tras `sintonia g module acta`:

```rust
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(user_router())
        .merge(project_router())
        .merge(acta_router())     // ← agregado por el CLI
        // sintonia:routes
}
```

---

## `sintonia check arch` — Guardián de la arquitectura — ADR 0001, 0008

```bash
sintonia check arch

Verificando jerarquía de crates...
  ✔ crates/domain — sin sqlx, sin axum, sin async-nats
  ✔ crates/application — sin sqlx, sin axum
  ✔ crates/database — único con sqlx ✓
  ✔ jsonwebtoken — no encontrado en ningún Cargo.toml

Verificando reglas de dominio...
  ✔ Soft Delete — sin DELETE directo en tabla users
  ✔ PASETO v4 configurado (pasetors)

Verificando tipos TypeScript...
  ✔ api.ts — marcado como generado, sin ediciones manuales

Estado: ✔ Arquitectura limpia
```

Fallo de ejemplo:

```bash
sintonia check arch

  ✗ crates/domain/Cargo.toml — imports sqlx: ERROR (regla 1 — ADR 0001)
  ✗ apps/api/Cargo.toml — imports jsonwebtoken: ERROR (JWT prohibido — ADR 0008)
  ✗ crates/database/src/repositories/user_repository.rs
    línea 47: DELETE FROM users — ERROR (usar UPDATE deleted_at — ADR 0006)

Estado: ✗ 3 violaciones detectadas — corregir antes de continuar
```

---

## Orden de implementación — ADR 0028

```
Fase 0 — ahora (OBLIGATORIO antes del CLI)
  Módulo 1: user    → 100% a mano (definir el patrón canónico)
  Módulo 2: project → 100% a mano (confirmar constantes)
  Módulo 3: report  → a mano con relaciones N:M (entender antes de automatizar)

Fase 1 — después del MVP
  sintonia g module → solo generación de módulos
  sintonia check arch → verificador de arquitectura
  sintonia db [migrate|seed|reset] → wrappers de sqlx-cli

Fase 2 — después del primer deploy en producción
  sintonia new <proyecto> → scaffolding completo del monorepo
  sintonia doctor → verificador de entorno

Fase 3 — cuando el sistema esté estable
  sintonia g module --with-events → módulo + evento NATS — ADR 0025
  sintonia g job <nombre> → worker Apalis — ADR 0018
  Modo interactivo con prompts
```

**El deploy va antes del CLI completo.** Tener algo en producción real vale más que
un CLI que genera código perfecto solo localmente.

---

## Flujo de trabajo real

```bash
# Día 1 — crear proyecto
sintonia new mi-saas
cd mi-saas
just setup

# Días 2-4 — módulos a mano (OBLIGATORIO — regla de los 3)
# Crear User manualmente — aprender el patrón (ADR 0028)
# Crear Project manualmente — confirmar el patrón
# Crear Report manualmente — entender relaciones N:M

# Después del deploy en producción — implementar el CLI (Fase 1)

# A partir del módulo 4 — el CLI trabaja por ti
sintonia g module factura
sintonia g module cliente
sintonia g module contrato --no-rbac  # sin permisos RBAC
sintonia g module pedido --dry-run    # ver qué crearía
```
