# ADR 0018 — Jobs Asíncronos: Apalis + SQLite

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (SQLite como backend de jobs en MVP), ADR 0025 (migración a NATS JetStream en Fase 2) |

---

## Contexto

Ciertas operaciones no deben bloquear la respuesta HTTP:

- Envío de emails de confirmación o notificación (ADR 0019)
- Generación de reportes en PDF
- Limpieza de tokens expirados, sesiones revocadas y audit_logs antiguos
- Llamadas a APIs externas (webhooks, integraciones)
- Procesamiento de archivos subidos

Sin un sistema de jobs, estas operaciones se ejecutan en el handler de Axum, alargando
la latencia percibida por el usuario y exponiendo el sistema a timeouts si la operación falla.

---

## Decisión

Usar **Apalis** con **SQLite como backend en MVP** — la misma base de datos del sistema principal,
sin ningún servicio externo adicional.

### Dependencias

```toml
# apps/api/Cargo.toml
apalis     = { version = "0.6", features = ["sqlite", "tracing", "retry"] }
apalis-sql = { version = "0.6", features = ["sqlite"] }
```

### Jobs del sistema

```rust
// apps/api/src/jobs/email_job.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailJob {
    pub to:       String,
    pub subject:  String,
    pub template: EmailTemplate,
    pub context:  serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EmailTemplate {
    Welcome,
    PasswordReset,
    LeadWelcome,
    Notification,
}

impl Job for EmailJob {
    const NAME: &'static str = "email::send";
}
```

```rust
// apps/api/src/jobs/cleanup_job.rs
// Limpia tokens expirados, sessions revocadas, refresh tokens y audit_logs >30 días
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupJob;

impl Job for CleanupJob {
    const NAME: &'static str = "maintenance::cleanup";
}

pub async fn handle_cleanup_job(_: CleanupJob, ctx: JobContext) -> Result<(), JobError> {
    let pool = ctx.data::<Arc<SqlitePool>>()?;

    sqlx::query!("DELETE FROM tokens WHERE expires_at < datetime('now')")
        .execute(&**pool).await?;
    sqlx::query!("DELETE FROM sessions WHERE expires_at < datetime('now')")
        .execute(&**pool).await?;
    sqlx::query!("DELETE FROM audit_logs WHERE created_at < datetime('now', '-30 days')")
        .execute(&**pool).await?;

    tracing::info!("cleanup completado");
    Ok(())
}
```

```rust
// apps/api/src/jobs/report_job.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct ReportJob {
    pub report_type: ReportType,
    pub user_id:     String,
    pub filters:     serde_json::Value,
}

impl Job for ReportJob {
    const NAME: &'static str = "report::generate";
}
```

### Configuración del worker

```rust
// apps/api/src/jobs/worker.rs
pub async fn start_workers(pool: Arc<SqlitePool>) -> Result<Monitor<SqliteStorage>, Box<dyn Error>> {
    let storage = SqliteStorage::new((*pool).clone());
    storage.setup().await?; // Crea las tablas de jobs si no existen

    let monitor = Monitor::new()
        .register_with_count(
            2, // 2 workers para emails — el más frecuente
            WorkerBuilder::new("email-worker")
                .data(pool.clone())
                .layer(RetryLayer::new(RetryPolicy::retries(3)))
                .layer(TraceLayer::new())
                .backend(SqliteStorage::<EmailJob>::new((*pool).clone()))
                .build_fn(handle_email_job),
        )
        .register(
            WorkerBuilder::new("cleanup-worker")
                .data(pool.clone())
                .layer(RetryLayer::new(RetryPolicy::retries(3)))
                .backend(SqliteStorage::<CleanupJob>::new((*pool).clone()))
                .build_fn(handle_cleanup_job),
        )
        .register(
            WorkerBuilder::new("report-worker")
                .data(pool.clone())
                .backend(SqliteStorage::<ReportJob>::new((*pool).clone()))
                .build_fn(handle_report_job),
        )
        .on_event(|e| tracing::debug!("worker event: {:?}", e));

    Ok(monitor)
}
```

### Encolar un job desde un caso de uso

```rust
// crates/application/src/use_cases/auth/register.rs
pub async fn execute(&self, input: RegisterInput) -> Result<RegisterOutput, DomainError> {
    // 1. Crear el usuario en DB
    let user = self.create_user_logic(input).await?;

    // 2. Encolar email — no bloquea la respuesta HTTP
    // Si falla el encole, el registro NO falla — solo se loggea el error
    if let Err(e) = self.job_storage.push(EmailJob {
        to:       user.email.clone(),
        subject:  "Bienvenido".to_string(),
        template: EmailTemplate::Welcome,
        context:  serde_json::json!({ "email": user.email }),
    }).await {
        tracing::error!(error = ?e, "failed to enqueue welcome email — non-fatal");
    }

    Ok(RegisterOutput { user_id: user.id })
}
```

### Diagnóstico de jobs fallidos

```sql
-- Ver jobs en estado Failed para inspección manual
SELECT id, job_type, attempts, last_error, updated_at
FROM jobs
WHERE status = 'Failed'
ORDER BY updated_at DESC
LIMIT 20;

-- Forzar reintento de un job fallido
UPDATE jobs SET status = 'Pending', attempts = 0 WHERE id = 'xxx';
```

---

## Path de migración cuando el volumen crezca

| Etapa | Backend | Cuándo migrar | Criterio concreto |
|-------|---------|---------------|-------------------|
| **MVP** | Apalis + SQLite | Hasta ~50 jobs/s | Jobs fallidos < 1% del total |
| **Escala media** | Apalis + PostgreSQL | >50 jobs/s sostenidos | Solo cambiar `SqliteStorage` por `PostgresStorage` — los workers no cambian |
| **Escala alta** | NATS JetStream | >1000 jobs/s o routing complejo | Ver ADR 0025 |

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| Redis + Sidekiq | Proceso externo — overhead innecesario en MVP |
| RabbitMQ | ~200MB de RAM — inviable en VPS de $5 |
| Tokio tasks puras | Sin durabilidad — si el proceso muere, los jobs se pierden |
| BullMQ (Node) | Fuera del stack Rust |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la fiabilidad y el monitoreo de las tareas de fondo:

| Herramienta | Propósito en el Procesamiento |
| :--- | :--- |
| **`apalis-cron`** | **Programación Nativa:** Permite definir cronogramas para tareas recurrentes (como limpieza) dentro de Rust. |
| **`sentry-apalis`** | **Reporte de Errores:** Captura fallos de jobs y los envía a Sentry con contexto completo de ejecución. |
| **`apalis-ui`** | **Dashboard:** Interfaz ligera para visualizar el estado de las colas y reintentar jobs desde la web. |
| **`typed-builder`** | **Ergonomía:** Facilita la construcción de payloads de jobs complejos de forma segura y legible. |

---

## Consecuencias

### ✅ Positivas

- Sin servidor adicional en MVP — usa la misma SQLite del sistema
- Durabilidad garantizada — los jobs persisten en DB aunque el proceso se reinicie
- Reintentos automáticos con backoff configurable (3 intentos por defecto)
- Tracing integrado — cada job tiene su span en los logs
- La respuesta HTTP no espera la operación asíncrona

### ⚠️ Negativas / Trade-offs

- Jobs en SQLite comparten el pool de conexiones con las queries normales
  → El pool tiene 10 conexiones; los workers usan máximo 4 (2 email + 1 cleanup + 1 report)
  → Las 6 conexiones restantes quedan para queries HTTP normales
  → Señal de contención: queries lentas en logs — ver ADR 0004 `log_slow_statements`
- Throughput limitado a ~50 jobs/s en SQLite
  → Suficiente para MVP — para comparar: 50 jobs/s = 4.3 millones de jobs/día
- Sin dashboard web nativo — jobs fallidos se inspeccionan en la DB
  → Query de diagnóstico documentada arriba
  → `CleanupJob` corre diariamente y limpia jobs completados >7 días
  → En Fase 2: activar el dashboard de Apalis (feature flag en `Cargo.toml`)

### Decisiones derivadas

- Los workers arrancan en el mismo proceso que la API — en Fase 2 separables como `apps/worker/`
- El email de bienvenida se encola en el caso de uso de registro, no en el handler HTTP
- El proveedor de email que ejecuta el `EmailJob` se define en ADR 0019
- Un job fallido tras 3 reintentos queda en estado `Failed` en la DB para inspección manual
- El `CleanupJob` también limpia tokens expirados, sessions revocadas y audit_logs >30 días
