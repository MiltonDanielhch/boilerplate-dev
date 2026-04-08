# ADR 0026 — NATS: Publisher + Infraestructura + Convención de Subjects

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado — implementación diferida a Fase 2 |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0017 (invalidación de caché L2), ADR 0018 (Apalis jobs), ADR 0025 (arquitectura de eventos) |

---

## Contexto

Este ADR consolida los detalles de implementación del publicador NATS y la configuración de
infraestructura, complementando la decisión arquitectónica del **ADR 0025**.

> **Referencia:** Ver ADR 0025 para el contexto arquitectónico, comparativa de buses de eventos,
> y justificación de NATS JetStream.

El sistema necesita desacoplar componentes y manejar tareas asíncronas sin bloquear el hilo
principal de la API de Rust.

---

## Decisión

Usar **NATS 2.10 con JetStream** como bus de eventos. Este ADR documenta los detalles de
implementación del publicador, la infraestructura (Docker/Podman) y la convención de subjects.

### Infraestructura Docker/Podman (Fase 2)

> **Nota:** Compatible con Docker y Podman (consistente con ADR 0014 — Deploy).

```yaml
# infra/docker/compose.nats.yml
services:
  nats:
    image: nats:2.10-alpine
    command: ["-js", "-sd", "/data", "-m", "8222"]
    volumes:
      - nats_data:/data
    ports:
      - "4222:4222"  # Client port
      - "8222:8222"  # HTTP monitoring — nats-top
    restart: unless-stopped

volumes:
  nats_data:
```

**Arrancar con Docker:**
```bash
docker compose -f infra/docker/compose.nats.yml up -d
```

**Arrancar con Podman (preferido — ADR 0014):**
```bash
podman compose -f infra/docker/compose.nats.yml up -d
# o con podman-compose
podman-compose -f infra/docker/compose.nats.yml up -d
```

### Publicador genérico en Rust

```rust
// crates/events/src/publisher.rs
use async_nats::jetstream::Context;
use serde::Serialize;

pub async fn publish_event<T: Serialize>(
    js:      &Context,
    subject: &str,
    payload: T,
) -> Result<(), EventError> {
    let bytes = serde_json::to_vec(&payload)
        .map_err(EventError::Serialization)?;

    // Publicación persistente con ACK explícito del servidor
    js.publish(subject.to_string(), bytes.into())
        .await
        .map_err(EventError::Publish)?
        .await  // Esperar ACK del servidor antes de continuar
        .map_err(EventError::Ack)?;

    Ok(())
}

// Uso tipado con constantes de subjects
pub struct EventPublisher {
    js: jetstream::Context,
}

impl EventPublisher {
    pub async fn user_registered(&self, user_id: &str, email: &str) -> Result<(), EventError> {
        publish_event(&self.js, subjects::USER_REGISTERED, serde_json::json!({
            "user_id": user_id,
            "email":   email,
            "version": "v1",
        })).await
    }
}
```

### Nomenclatura de subjects — inamovible

```rust
// crates/events/src/subjects.rs
// Constantes en lugar de strings literales — el compilador detecta typos
pub const USER_REGISTERED:   &str = "boilerplate.user.registered.v1";
pub const REPORT_GENERATED:  &str = "boilerplate.report.generated.v1";
pub const EMAIL_QUEUED:      &str = "boilerplate.email.queued.v1";
pub const SESSION_EXPIRED:   &str = "boilerplate.session.expired.v1";
```

Formato: `boilerplate.{dominio}.{evento}.{version}`

Cambiar el prefijo `boilerplate.*` rompe todos los consumidores existentes — es inamovible.

### Herramientas de diagnóstico

```bash
# Instalar CLI de NATS
go install github.com/nats-io/natscli/nats@latest

# Ver estado del stream
nats stream info BOILERPLATE_EVENTS

# Ver consumidores y su lag
nats consumer info BOILERPLATE_EVENTS email_worker

# Ver mensajes pendientes
nats stream get BOILERPLATE_EVENTS --seq 1

# Monitoreo en tiempo real
nats-top  # go install github.com/nats-io/nats-top@latest
```

---

## Herramientas Recomendadas

| Herramienta | Propósito |
|-------------|-----------|
| `nats-cli` | CLI oficial para diagnóstico de streams y consumidores |
| `nats-top` | Monitoreo en tiempo real de throughput por subject |
| `opentelemetry-nats` | Propagación de contexto distribuido (trace_id) |

---

## Consecuencias

### ✅ Positivas

- Latencia de microsegundos — adecuado para operaciones en tiempo real
- Mensajes persistidos en disco — si el consumidor se cae, los mensajes esperan
- Un solo binario sin dependencias externas
- `async-nats` se integra nativamente con el modelo async de Rust

### ⚠️ Negativas / Trade-offs

- Menos herramientas visuales que Kafka
  → `nats stream info` y `nats consumer info` son suficientes para diagnóstico
  → `nats-top` para monitoreo en tiempo real
  → En Fase 3: NATS surveyor + Grafana dashboard si se necesita visualización avanzada
- Los consumidores deben confirmar con `msg.ack()` — sin ACK el mensaje se reenvía
  → At-least-once delivery es el comportamiento correcto para este sistema
  → Los consumidores deben ser idempotentes — usar el `user_id` como clave de deduplicación
  → Ver ADR 0025 para la configuración del consumidor durable

### Decisiones derivadas

- El prefijo de subjects `boilerplate.*` se define como constantes — nunca strings literales
- Los logs del sistema pueden enviarse a un stream dedicado para análisis posterior
- Ver ADR 0025 para la configuración completa del stream JetStream y consumidores durables
