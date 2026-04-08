# ADR 0028 — Sintonía CLI: Generador con RBAC

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado — implementación en Fase 2 |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Monolito Modular), ADR 0005 (Migraciones), ADR 0006 (RBAC + Sessions + Audit) |

---

## Contexto

La arquitectura hexagonal con DDD es potente pero requiere crear múltiples archivos con
estructuras repetitivas por cada módulo: entidad, port, adaptador, handler, DTO, migración,
tests y permisos RBAC. Hacer esto a mano es lento y propenso a inconsistencias.

Necesitamos una herramienta que:

- **Estandarice** — cada módulo tiene exactamente la misma estructura
- **Acelere** — reducir la creación de un CRUD completo de horas a segundos
- **Conozca el sistema** — sabe que existe RBAC, Soft Delete y auditoría

---

## Decisión

Desarrollar **Sintonía CLI** en Rust con `clap`, construido **después de haber creado
3 módulos a mano**.

### La regla de los 3 módulos — inamovible

El CLI automatiza patrones que ya se entienden. No al revés.

| Módulo | Cómo | Propósito |
|--------|------|-----------|
| `user` | 100% a mano | Definir el patrón canónico con RBAC |
| `project` | 100% a mano | Confirmar qué partes son constantes |
| `report` | A mano con relaciones | Entender N:M antes de automatizarlo |
| `módulo 4...N` | `sintonia g module` | El CLI genera el patrón ya conocido |

### Arquitectura del CLI

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
    │   └── db.rs            # sintonia db migrate | seed | reset
    ├── generators/
    │   ├── module.rs        # Orquesta todos los sub-generadores
    │   ├── entity.rs
    │   ├── repository.rs
    │   ├── use_case.rs
    │   ├── handler.rs
    │   ├── migration.rs
    │   └── rbac.rs          # Genera permisos en el seed
    ├── templates/
    │   ├── entity.rs.tera
    │   ├── port.rs.tera
    │   ├── use_case.rs.tera
    │   ├── handler.rs.tera
    │   └── migration.sql.tera
    └── utils/
        ├── naming.rs        # user → User → users → user_id
        ├── fs.rs            # Idempotencia — no sobrescribir si existe
        └── ast_editor.rs    # Modificar archivos con marcadores
```

### Lo que genera `sintonia g module acta`

```
crates/domain/src/entities/acta.rs         # struct con Soft Delete
crates/domain/src/ports/acta_repository.rs # trait con has_permission

crates/database/src/repositories/
  sqlite_acta_repository.rs                # impl con query_as!
  cached_acta_repository.rs               # Decorator Moka (ADR 0017)

crates/application/src/use_cases/
  create_acta.rs
  get_acta.rs
  list_actas.rs
  soft_delete_acta.rs

crates/infrastructure/src/http/handlers/
  acta_handler.rs                          # con #[utoipa::path] (ADR 0021)
  acta_dto.rs

data/migrations/{timestamp}_create_actas.sql  # Soft Delete + trigger

# RBAC — permisos agregados al seed automáticamente:
INSERT OR IGNORE INTO permissions (id, name, description) VALUES
('perm_acta_001', 'actas:read',  'Ver lista de actas'),
('perm_acta_002', 'actas:write', 'Crear y editar actas');
INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.name = 'Admin' AND p.name LIKE 'actas:%';

# Registros automáticos en archivos existentes:
crates/infrastructure/src/http/router.rs ← // sintonia:routes
crates/domain/src/lib.rs                 ← // sintonia:modules
```

### Marcadores en archivos existentes

```rust
// crates/infrastructure/src/http/router.rs
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(user_router())
        .merge(project_router())
        // sintonia:routes  ← el CLI inserta aquí — nunca eliminar
}

// crates/domain/src/lib.rs
pub mod entities;
pub mod ports;
// sintonia:modules  ← el CLI inserta aquí — nunca eliminar
```

### Naming inteligente

```
entrada → singular → plural → id_field → DTO
acta    → Acta     → actas  → acta_id  → ActaDto
post    → Post     → posts  → post_id  → PostDto
```

### `sintonia check arch` — Guardián de la arquitectura

```bash
sintonia check arch

Verificando jerarquía de crates...
  ✔ crates/domain — sin sqlx, sin axum
  ✔ crates/database — único con sqlx ✓
  ✔ jsonwebtoken — no encontrado en ningún Cargo.toml
  ✔ PASETO v4 configurado (pasetors)

Verificando reglas de dominio...
  ✔ Sin DELETE directo en tabla users (Soft Delete activo)
  ✔ api.ts marcado como generado

Estado: ✔ Arquitectura limpia
```

Error cuando hay violación:

```bash
  ✗ crates/domain/Cargo.toml importa sqlx — ERROR (regla 1)
  ✗ jsonwebtoken en apps/api/Cargo.toml — ERROR (JWT prohibido)
  ✗ DELETE FROM users en line 47 — ERROR (usar UPDATE deleted_at)
```

---

## Comandos del CLI

```bash
# Proyecto
sintonia new <proyecto>           # scaffolding monorepo completo
sintonia doctor                   # verifica entorno y herramientas

# Generación de módulos
sintonia g module <nombre>        # módulo completo con RBAC
sintonia g module <nombre> --no-rbac  # sin permisos (entidades internas)
sintonia g module <nombre> --dry-run  # muestra qué crearía sin tocar nada
sintonia g entity <nombre>        # solo la entidad de dominio
sintonia g migration <nombre>     # solo la migración SQL

# Base de datos
sintonia db migrate               # sqlx migrate run
sintonia db seed                  # ejecuta los seeds
sintonia db reset                 # borra y recrea

# Verificación
sintonia check arch               # detecta imports prohibidos entre crates
```

---

## Orden de implementación

```
Fase 0 (ahora)       → 3 módulos a mano — user, project, report
Fase 1 (post-MVP)    → sintonia g module — solo generación
Fase 2 (escala)      → sintonia new + db + check arch
```

El deploy va antes del CLI completo — el CLI sin producción es optimizar algo que
todavía no se sabe si funciona correctamente.

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| `cargo-generate` | Sin conocimiento del RBAC ni de las convenciones del proyecto |
| Scripts bash | Sin tipado, difíciles de mantener, sin AST editing |
| Yeoman / Plop | Requiere Node.js — fuera del stack Rust |
| Construir CLI primero | Automatiza patrones que no se entienden todavía |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la eficiencia y la calidad de la Sintonía CLI:

| Herramienta | Propósito en el CLI |
| :--- | :--- |
| **`clap_mangen`** | **Documentación:** Genera páginas `man` automáticamente para los comandos del CLI. |
| **`syn` / `quote`** | **Edición de AST:** Para manipular código Rust de forma segura y programática (más allá de marcadores). |
| **`comfy-table`** | **Salida legible:** Formatea la salida de comandos como `check arch` en tablas claras y atractivas. |
| **`walkdir`** | **Recorrido de Archivos:** Para explorar directorios de forma eficiente en los generadores. |

---

## Consecuencias

### ✅ Positivas

- Todos los módulos tienen exactamente la misma estructura — código predecible
- Los permisos RBAC se generan automáticamente — imposible olvidarlos
- Los tests se generan junto con el módulo — cobertura mínima garantizada
- El Soft Delete se aplica por defecto — nunca un DELETE accidental

### ⚠️ Negativas / Trade-offs

- El CLI es un proyecto en sí mismo que requiere mantenimiento
  → Está en `apps/cli/` como cualquier otra app del monorepo — mismas reglas de calidad
- Riesgo de que el developer olvide cómo funciona el sistema bajo el capó
  → Mitigado con la regla de los 3 módulos manuales obligatorios antes de usar el CLI

### Decisiones derivadas

- Los marcadores `// sintonia:routes` y `// sintonia:modules` son inamovibles — nunca eliminar
- `sintonia g module` genera tests unitarios y de integración por defecto
- `sintonia check arch` se integra en el pre-push hook de lefthook (ADR 0012)
- El CLI vive en `apps/cli/` — no en `tools/` ni en una carpeta separada
