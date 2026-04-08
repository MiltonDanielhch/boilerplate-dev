# ADR 0025 — Eventos: NATS JetStream (Fase 2)

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado — implementación diferida a Fase 2 |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0018 (Apalis como sistema de jobs en Fase 1), ADR 0004 (Persistencia SQLite) |

---

## Contexto

En Fase 2, cuando el sistema requiera comunicación asíncrona garantizada entre módulos o
servicios, necesitaremos un bus de eventos que:

- **Persista mensajes en disco** — si el consumidor se cae, el mensaje espera
- **Consuma mínima RAM** — no más de 50MB en reposo
- **Se integre nativamente con Rust** — tipado fuerte en los mensajes

Kafka y RabbitMQ quedan descartados por consumo de RAM en un VPS de $5. NATS con JetStream
resuelve ambas restricciones en un único binario de ~25MB.

---

## Decisión

Usar **NATS 2.10** con **JetStream habilitado** para persistencia de eventos en disco.

### Infraestructura (Fase 2 — `infra/docker/compose.nats.yml`)

```yaml
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

### Nomenclatura de subjects — inamovible

```
boilerplate.{dominio}.{evento}.{version}

Ejemplos:
  boilerplate.user.registered.v1
  boilerplate.report.generated.v1
  boilerplate.email.queued.v1
  boilerplate.session.expired.v1
```

Cambiar el prefijo `boilerplate.*` rompe todos los consumidores existentes.

### Inicialización del stream en Rust

```rust
// crates/events/src/jetstream.rs
use async_nats::jetstream;

pub async fn setup_jetstream(
    client: async_nats::Client,
) -> Result<jetstream::Context, EventError> {
    let js = jetstream::new(client);

    js.get_or_create_stream(jetstream::stream::Config {
        name:      "BOILERPLATE_EVENTS".to_string(),
        subjects:  vec!["boilerplate.>".to_string()],
        storage:   jetstream::stream::StorageType::File,
        retention: jetstream::stream::RetentionPolicy::Limits,
        max_age:   Duration::from_secs(60 * 60 * 24 * 7), // Retención 7 días
        max_bytes: 1_073_741_824, // 1GB máximo en disco
        ..Default::default()
    })
    .await?;

    Ok(js)
}
```

### Componentes del sistema de eventos

| Componente | Responsabilidad | Documentación |
|------------|-----------------|---------------|
| **Publicador** | Enviar eventos al stream con ACK | ADR 0026 — Implementación |
| **Consumidor durable** | Recibir eventos con garantía de entrega | ADR 0026 — Implementación |
| **Subjects** | Nomenclatura de canales | Ver abajo |
| **Infraestructura** | Docker Compose para NATS | ADR 0026 — Docker |

> **Nota:** Ver ADR 0026 para el código completo del publicador, consumidor, y configuración de infraestructura Docker.

---

## Comparativa de buses de eventos para VPS $5

| Bus | RAM en reposo | Persistencia | Complejidad operativa |
|-----|--------------|-------------|----------------------|
| Redis Pub/Sub | ~10MB | ❌ Sin ACK | Muy baja |
| RabbitMQ | ~200MB+ | ✅ | Alta |
| Kafka | ~500MB+ | ✅ | Muy alta (JVM + ZooKeeper) |
| **NATS JetStream** | **~25MB** | **✅ en disco** | **Baja — un solo binario** |

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| Redis Pub/Sub | Sin garantía de entrega — si el consumidor está caído, el mensaje se pierde |
| RabbitMQ | ~200MB de RAM — inviable en VPS de $5 |
| Kafka | >500MB de RAM, requiere ZooKeeper o KRaft — demasiado para un monolito |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para industrializar el manejo de eventos y asegurar la interoperabilidad:

| Herramienta | Propósito en el Bus de Eventos |
| :--- | :--- |
| **`cloudevents-sdk`** | **Estándar de Eventos:** Provee un sobre (envelope) común para todos los mensajes, facilitando la auditoría y el trazado. |
| **`nats-dashboard`** | **Visualización:** Interfaz web ligera para monitorear el estado de los streams y consumidores en tiempo real. |
| **`rkyv`** | **Serialización Ultra-rápida:** Alternativa a JSON para alto rendimiento con cero copias en memoria. |
| **`nats-server` (Leaf Nodes)** | **Sincronía Edge:** Permite extender el bus de eventos hacia aplicaciones móviles o desktop (ADR 0030). |

---

## Consecuencias

### ✅ Positivas

- Los eventos sobreviven a caídas del sistema — JetStream persiste en disco
- NATS es el bus más ligero que admite persistencia real (~25MB en reposo)
- Se pueden agregar nuevos consumidores sin tocar los productores existentes
- Path claro a escala: añadir réplicas de NATS sin cambiar el código de la app

### ⚠️ Negativas / Trade-offs

- Los consumidores deben confirmar explícitamente con `msg.ack()` — sin ACK el mensaje se reenvía
  → Sin ACK es la garantía de at-least-once delivery — los consumidores deben ser idempotentes
  → El `durable_name` garantiza que NATS recuerda la posición del consumidor entre reinicios
- El directorio de JetStream puede llenar el disco (retención de 7 días configurada)
  → Configurar límite de tamaño en el stream: `max_bytes: 1_073_741_824` (1GB)
  → Monitorear con `nats-top` o la CLI:
    `nats stream info BOILERPLATE_EVENTS`
    `nats consumer info BOILERPLATE_EVENTS email_worker`
  → Alerta en CleanupJob si `/data/nats/` supera 80% del disco disponible
- Un sujeto mal formado no genera error en tiempo de compilación
  → Mitigación: definir constantes para los subjects:
    `pub const USER_REGISTERED: &str = "boilerplate.user.registered.v1";`

### Decisiones derivadas

- La nomenclatura `boilerplate.{dominio}.{evento}.{version}` es inamovible
- Se usará JSON como serialización en Fase 2 para facilitar debugging
- Fase 3 migra a Protobuf (ADR 0027) cuando la eficiencia binaria sea necesaria
- El volumen `nats_data/` se incluye en el snapshot del VPS
- Ver ADR 0026 para detalles del publicador y la infraestructura NATS
