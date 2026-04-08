# ADR 0005 — Migraciones SQLx y Seeding Idempotente

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (SQLite + SQLx), ADR 0001 (Monolito Modular) |

---

## Contexto

A medida que el proyecto crece, modificar la base de datos manualmente es propenso a errores
y rompe la consistencia entre entornos. Necesitamos un sistema que:

- **Automatice los cambios** — las tablas se crean o modifican al deployar, sin intervención manual
- **Versionado de esquema** — saber exactamente en qué versión de DB está cada entorno
- **Datos de prueba reproducibles** — poblar la DB con datos ficticios con un solo comando

---

## Decisión

Usar **SQLx migrations** para la evolución del esquema y una **CLI de seeds en Rust** para
los datos iniciales de desarrollo.

### 1 — Migraciones con SQLx

Las migraciones viven en `data/migrations/`. Cada archivo es inmutable una vez ejecutado
en producción — para corregir, se crea una nueva migración:

```bash
# Crear nueva migración — genera el archivo con timestamp
sqlx migrate add crear_tabla_actas
# → data/migrations/20260324000001_crear_tabla_actas.sql

# Ejecutar migraciones pendientes (automático al arrancar el servidor)
sqlx migrate run

# Ver estado de migraciones
sqlx migrate info
```

```sql
-- data/migrations/20260324000001_crear_tabla_actas.sql
CREATE TABLE IF NOT EXISTS actas (
    id         TEXT     PRIMARY KEY NOT NULL,
    titulo     TEXT     NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME           -- Soft Delete — NULL = activo
);

CREATE INDEX IF NOT EXISTS idx_actas_active
ON actas(id) WHERE deleted_at IS NULL;

CREATE TRIGGER IF NOT EXISTS trg_actas_updated_at
AFTER UPDATE ON actas FOR EACH ROW
BEGIN
    UPDATE actas SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;
```

### 2 — Migraciones automáticas al arrancar

```rust
// apps/api/src/main.rs
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let pool = create_pool(&config.database_url).await?;

    // El servidor no arranca si las migraciones fallan — fail-fast intencional
    // Si falla → Kamal detecta el healthcheck fallido y hace rollback (ADR 0014)
    sqlx::migrate!("../../data/migrations")
        .run(&pool)
        .await
        .expect("❌ migraciones fallaron — revisar data/migrations/");

    // ...iniciar servidor
}
```

### 3 — Seeding para desarrollo

Los seeds no contaminan el binario de producción — viven como subcomando del CLI:

```rust
// apps/cli/src/commands/seed.rs
pub async fn seed_development(pool: &SqlitePool) -> Result<(), AppError> {
    // Idempotente — no falla si ya existen los datos
    let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    if count > 0 {
        tracing::info!("DB ya tiene datos — omitiendo seed");
        return Ok(());
    }

    // Usar INSERT OR IGNORE — las 6 migraciones base (ADR 0006) ya incluyen el seed del admin
    sqlx::query!(
        "INSERT OR IGNORE INTO users (id, email, password_hash, email_verified) VALUES (?, ?, ?, 1)",
        Uuid::new_v4().to_string(),
        "dev@boilerplate.dev",
        hash_password("dev_password_123")?,
    )
    .execute(pool)
    .await?;

    tracing::info!("seed de desarrollo completado");
    Ok(())
}
```

```makefile
# justfile
db-reset:
    sqlx database reset        # Borra y recrea la DB
    just migrate               # Corre las 6 migraciones base
    cargo run --bin cli seed   # Inserta datos de prueba adicionales
```

### 4 — Checklist antes de commitear una migración

```bash
# 1. Correr la migración en desarrollo
just migrate                           # Debe completar sin errores

# 2. Verificar que es idempotente
just db-reset && just migrate          # Debe completar igual

# 3. Verificar integridad
sqlite3 ./data/boilerplate.db "PRAGMA integrity_check;"  # → ok

# 4. Actualizar .sqlx/ para SQLX_OFFLINE
just prepare                           # Actualiza .sqlx/ en git
git add .sqlx/
```

### 5 — Proceso de corrección de migraciones en producción

Si una migración ejecutada tiene un error:

```bash
# ❌ NUNCA modificar el archivo de migración existente
# ✅ Crear una nueva migración que corrija el problema

sqlx migrate add corregir_nombre_tabla_actas
# Editar el nuevo archivo con el ALTER TABLE o DROP/CREATE correspondiente
```

---

## Comparativa: SQLx vs ORMs

| Característica | SQLx | SeaORM / Diesel |
|----------------|------|-----------------|
| **Control del SQL** | Total — SQL puro | Abstracción en Rust |
| **Rendimiento** | Máximo — sin overhead | Ligero overhead |
| **Seguridad** | Query checks en compilación | Tipado fuerte en Rust |
| **Curva de aprendizaje** | Requiere saber SQL | Menor — pero oculta el SQL |

Para este proyecto, la visibilidad y control del SQL son prioritarios.

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la eficiencia en la gestión del esquema y datos de prueba:

| Herramienta | Propósito en Migraciones / Seeding |
| :--- | :--- |
| **`fake`** | **Datos Realistas:** Generación de nombres, emails y fechas aleatorias pero coherentes para los seeds. |
| **`cargo-run-bin`** | **Versionado de CLI:** Garantiza que todo el equipo use la misma versión de `sqlx-cli` sin instalaciones globales. |
| **`sqldef`** | **Declarative SQL:** (Opcional) Permite definir el estado deseado y genera el diff de SQL automáticamente. |
| **`just`** | **Automatización:** Orquestador de tareas que unifica `reset`, `migrate` y `prepare` en comandos simples. |

---

## Consecuencias

### ✅ Positivas

- Deploy atómico — el servidor no arranca si las migraciones fallan
- El historial de migraciones documenta la evolución del esquema — auditable en git
- `just db-reset` deja un entorno limpio con datos de prueba en segundos
- `SQLX_OFFLINE=true` permite compilar sin DB — las queries se verifican contra `.sqlx/`

### ⚠️ Negativas / Trade-offs

- SQL manual — requiere conocer DDL para crear tablas y alterar columnas
  → El CLI (ADR 0028) genera la migración automáticamente para módulos nuevos
  → `sintonia g migration <nombre>` crea el archivo con la estructura correcta
- Las migraciones ejecutadas en producción son inmutables
  → Ver sección "Proceso de corrección" arriba — crear nueva migración para corregir
  → Nunca modificar un archivo de migración ya ejecutado
- `just prepare` debe correr antes de commitear cambios de SQL
  → Incluido en el pre-push hook de lefthook (ADR 0012)
  → El CI falla con error claro si `.sqlx/` no está actualizado

### Decisiones derivadas

- Las migraciones viven en `data/migrations/` — separadas del código de la app
- Los seeds solo corren en entornos `development` y `test` — nunca en producción automáticamente
- Las 6 migraciones base del ADR 0006 son el punto de partida — no se modifican
- `just prepare` está en el pre-push hook de lefthook (ADR 0012)
