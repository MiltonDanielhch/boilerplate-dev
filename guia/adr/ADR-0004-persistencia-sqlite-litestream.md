# ADR 0004 — Persistencia: SQLite WAL + SQLx + Litestream

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0018 (Apalis usa SQLite como backend en MVP) |

---

## Contexto

Necesitamos una estrategia de persistencia que cumpla con:

- **Costo operativo cero** en infraestructura de base de datos — sin servidor separado
- **Consultas seguras en compile-time** — los errores de SQL se detectan antes de ejecutar
- **Backups automáticos** sin scripts manuales ni ventanas de mantenimiento
- **Baja latencia** en lecturas — la mayoría del tráfico del sistema es read-heavy
- **Funciona en un VPS de $5** sin comprometer durabilidad

La alternativa natural era PostgreSQL, pero requiere un servidor adicional, configuración de
réplicas para HA, y aumenta la complejidad operativa desde el día uno. SQLite con WAL mode
resuelve los casos de uso del MVP y tiene un path de migración claro cuando el volumen lo justifique.

---

## Decisión

Usar **SQLite en modo WAL** con **SQLx** para acceso a datos y **Litestream** para replicación
continua hacia S3.

### Configuración SQLite optimizada

```sql
-- Aplicar al crear el pool — PRAGMAs en crates/database/src/pool.rs
PRAGMA journal_mode = WAL;         -- Escrituras y lecturas simultáneas sin bloqueo
PRAGMA synchronous  = NORMAL;      -- Durabilidad con mejor rendimiento (vs FULL)
PRAGMA temp_store   = MEMORY;      -- Tablas temporales en RAM
PRAGMA mmap_size    = 30000000000; -- 30GB de memory-mapped I/O
PRAGMA foreign_keys = ON;          -- Integridad referencial activada siempre
PRAGMA cache_size   = -64000;      -- 64MB de page cache
```

### Pool de conexiones

```rust
// crates/database/src/pool.rs
pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(10)           // Límite: SQLite no escala con muchos writers
        .min_connections(2)            // Mantener conexiones calientes
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .connect_with(
            SqliteConnectOptions::from_str(database_url)?
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .pragma("temp_store",   "MEMORY")
                .pragma("mmap_size",    "30000000000")
                .pragma("foreign_keys", "ON")
                .pragma("cache_size",   "-64000")
                .log_statements(LevelFilter::Debug)
                .log_slow_statements(LevelFilter::Warn, Duration::from_millis(100)),
        )
        .await
}
```

### Ejemplo de query con SQLx

```rust
// Queries chequeadas en compile-time con query_as!
// Si la columna no existe o el tipo no coincide → error de compilación
let user = sqlx::query_as!(
    UserRow,
    r#"
    SELECT id, email, password_hash, email_verified,
           created_at AS "created_at: OffsetDateTime",
           deleted_at AS "deleted_at: Option<OffsetDateTime>"
    FROM users
    WHERE id = ? AND deleted_at IS NULL
    "#,
    user_id
)
.fetch_optional(&pool)
.await?;
```

### Configuración de Litestream

```yaml
# infra/litestream/litestream.yml
dbs:
  - path: /data/boilerplate.db
    replicas:
      - type: s3
        bucket:             ${LITESTREAM_BUCKET}
        path:               boilerplate/db
        region:             ${AWS_REGION}
        access-key-id:      ${AWS_ACCESS_KEY_ID}
        secret-access-key:  ${AWS_SECRET_ACCESS_KEY}
        endpoint:           ${AWS_ENDPOINT_URL_S3}  # Tigris endpoint
        sync-interval:      1s    # RPO efectivo ~1 segundo
        snapshot-interval:  24h   # Snapshot diario completo
        retention:          72h   # Retener 3 días de WAL
```

### Litestream como sidecar en el Containerfile

```dockerfile
# infra/docker/Containerfile
FROM gcr.io/distroless/cc-debian12

COPY --from=ghcr.io/benbjohnson/litestream:latest-amd64 \
    /usr/local/bin/litestream /litestream
COPY litestream.yml /etc/litestream.yml

# Litestream wrappea el proceso principal — replica mientras la API corre
ENTRYPOINT ["/litestream", "replicate", "-exec", "/api"]
```

---

## Path de migración cuando el volumen crezca

| Etapa | Tecnología | Cuándo migrar | Criterio concreto |
|-------|-----------|---------------|-------------------|
| **MVP** | SQLite WAL + Litestream | Hasta ~100 writes/s | Latencia P99 < 100ms en /health |
| **Escala media** | Turso (SQLite distribuido) | Multi-nodo o >100 writes/s | Solo cambiar la cadena de conexión — misma API SQLx |
| **Escala alta** | PostgreSQL | Joins analíticos o réplicas de lectura | Solo cambia el adaptador en `crates/database` — el dominio no se toca |

Ver ADR 0031 para los criterios completos de activación de cada nivel.

---

## Alternativas consideradas

| Opción | Motivo de descarte en MVP |
|--------|--------------------------|
| PostgreSQL | Servidor adicional, complejidad operativa innecesaria en MVP |
| MySQL | Sin query checking en compile-time tan maduro como SQLx |
| SurrealDB | Ecosistema inmaduro, riesgo alto |
| EdgeDB | Overhead de aprendizaje alto sin beneficio claro en MVP |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para extender las capacidades de SQLite y facilitar la evolución del esquema:

| Herramienta | Propósito en la Persistencia |
| :--- | :--- |
| **`sqlite-vec`** | **IA & Vector Search:** Permite realizar búsquedas semánticas y embeddings directamente en SQLite sin servicios externos. |
| **`libsql`** | **Escalabilidad:** El motor detrás de Turso; permite replicación geográfica si el MVP crece fuera de un solo nodo. |
| **`sea-query`** | **Queries Dinámicas:** Constructor de SQL type-safe para casos donde `query_as!` es demasiado rígido (filtros dinámicos). |
| **`sqlx-cli`** | **Gestión de Migraciones:** Herramienta esencial para el flujo de CI/CD y validación de esquemas en frío. |

---

## Consecuencias

### ✅ Positivas

- Cero servidor de base de datos que mantener
- Lecturas con latencia < 1ms — los datos viven en el mismo proceso
- Backups continuos automáticos — RPO efectivo de ~1 segundo
- Queries verificadas en compile-time — imposible hacer un typo de columna en producción
- WAL mode: readers y writers no se bloquean entre sí

### ⚠️ Negativas / Trade-offs

- Máximo ~10 conexiones concurrentes — limitación de SQLite con múltiples writers
  → Las 10 conexiones son suficientes para ~500 usuarios activos simultáneos porque
    cada request tarda <10ms en SQLite con WAL
  → Señal de que hay que migrar: latencia P99 >100ms en `/health` durante >30 minutos
    Ver ADR 0031 Nivel 2 para el proceso de migración a Turso
- No es adecuado para escrituras muy concurrentes (>100 writes/s simultáneos)
  → Apalis maneja la contención de jobs con su propio mecanismo — no necesita
    `SKIP LOCKED` porque usa SQLite con row-level locking (ADR 0018)
- No hay soporte nativo para `SKIP LOCKED`
  → No es necesario en este stack — Apalis resuelve la contención de colas sin él

### Decisiones derivadas

- Apalis usa SQLite como backend de jobs en MVP → ver **ADR 0018**
- El pool se comparte vía `Arc<SqlitePool>` a través del `AppState`
- Litestream corre como sidecar en el mismo contenedor que el binario principal
- `just prepare` debe correr antes de commitear cambios de SQL — actualiza `.sqlx/` para `SQLX_OFFLINE=true`
