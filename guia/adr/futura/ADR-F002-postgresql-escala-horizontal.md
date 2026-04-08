# ADR F-002 — Futuro: PostgreSQL + Escala Horizontal

| Campo | Valor |
|-------|-------|
| **Estado** | 🔮 Futuro — activar en ADR 0031 Nivel 2b |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (SQLite WAL — persistencia actual), ADR 0031 (Escalamiento Nivel 2b) |

---

## Contexto

Este ADR documenta el path de migración a PostgreSQL cuando SQLite WAL y Turso
ya no sean suficientes para el volumen de escrituras o cuando se necesiten
réplicas de lectura en múltiples regiones.

**Activar cuando (ADR 0031 Nivel 2b):**
- Joins analíticos complejos que SQLite no maneja bien
- Necesidad de réplicas de lectura dedicadas
- >100.000 usuarios activos con escrituras concurrentes pesadas
- Herramientas BI que requieren PostgreSQL específicamente

---

## Decisión futura

Migrar `crates/database` para usar **PostgreSQL** manteniendo la misma API de repositorios.

### El cambio es quirúrgico gracias a ADR 0001

```rust
// crates/database/src/pool.rs — EL ÚNICO ARCHIVO QUE CAMBIA
// De SQLite:
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
pub async fn create_pool(url: &str) -> SqlitePool {
    SqlitePoolOptions::new().connect(url).await.unwrap()
}

// A PostgreSQL:
use sqlx::postgres::{PgPool, PgPoolOptions};
pub async fn create_pool(url: &str) -> PgPool {
    PgPoolOptions::new().connect(url).await.unwrap()
}
```

### Cambios en queries — mínimos

SQLx soporta ambos dialectos. Los cambios principales son:

| SQLite | PostgreSQL |
|--------|-----------|
| `?` como placeholder | `$1`, `$2` como placeholder |
| `datetime('now')` | `NOW()` |
| `TEXT` para UUID | `UUID` nativo |
| `BOOLEAN` como INTEGER | `BOOLEAN` nativo |

### Infraestructura PostgreSQL

```yaml
# infra/docker/compose.postgres.yml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB:       boilerplate
      POSTGRES_USER:     boilerplate
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  # Réplica de lectura (cuando sea necesaria)
  postgres_replica:
    image: postgres:16-alpine
    environment:
      POSTGRES_MASTER_SERVICE: postgres
    depends_on:
      - postgres
```

### Litestream → pg_basebackup

Litestream solo replica SQLite. Al migrar a PostgreSQL:

```bash
# Backup con pg_basebackup
pg_basebackup -h localhost -U boilerplate -D /backup/postgres -Ft -z -P

# O con Barman para backups continuos
barman backup main
```

### Estrategia de migración de datos

```bash
# 1. Exportar datos de SQLite
sqlite3 ./data/boilerplate.db .dump > backup.sql

# 2. Adaptar el SQL dump para PostgreSQL
# (cambiar tipos, placeholders, funciones)

# 3. Importar en PostgreSQL
psql -U boilerplate -d boilerplate < backup_adapted.sql

# 4. Verificar integridad
# Comparar conteos de filas entre SQLite y PostgreSQL
```

---

## Estado actual

No implementar. Mantener SQLite WAL (ADR 0004) hasta que los criterios del ADR 0031
Nivel 2b sean evidentes en datos de producción reales.

La transición es segura porque la arquitectura hexagonal del ADR 0001 garantiza
que el dominio, los casos de uso y los tests nunca necesitan modificarse.
