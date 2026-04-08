# Roadmap — Fase 2: NATS + Eventos + Workers

> **Objetivo:** Implementar NATS JetStream para desacoplar el API HTTP de los workers pesados.
> Esto permite que jobs pesados (reportes, emails masivos, procesamiento de datos) no degraden la latencia del API.
>
> **Stack:** NATS 2.10 · JetStream · async-nats
>
> **ADRs:** 0025 (Arquitectura de Eventos), 0026 (Publisher + Infraestructura), 0018 (Apalis Jobs)
>
> **Criterio de activación:** latencia P95 >50ms cuando Apalis procesa jobs pesados en el mismo proceso.
> NO implementar sin criterio medido en producción real.

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Fase | Nombre | Progreso |
|------|--------|----------|
| F2.1 | Infraestructura NATS | 0% |
| F2.2 | Crate events + Publisher | 0% |
| F2.3 | Consumidor durable + Worker | 0% |
| F2.4 | Separación API vs Worker | 0% |
| F2.5 | Verificaciones | 0% |

---

## F2.1 — Infraestructura NATS

> **Referencia:** ADR 0025 (Arquitectura), ADR 0026 (Infraestructura), docs/02-STACK.md L172-177

```
[ ] Verificar que el criterio existe con datos reales antes de empezar
    └─ Ref: ADR 0025 — Fase 2 solo cuando el problema sea real

[ ] Crear infra/docker/compose.nats.yml:
    └─ Ref: ADR 0026, docs/03-STRUCTURE.md L571-573
    [ ] services: nats con image: nats:2.10-alpine
    [ ] command: ["-js", "-sd", "/data", "-m", "8222"]
    [ ] volumes: nats_data:/data
    [ ] ports: 4222:4222, 8222:8222

[ ] Arrancar NATS:
    [ ] Docker: docker compose -f infra/docker/compose.nats.yml up -d
    [ ] Podman (preferido): podman compose -f infra/docker/compose.nats.yml up -d
        └─ Ref: ADR 0014 — Podman es el runtime preferido

[ ] Verificar NATS está funcionando:
    [ ] nats stream info → conecta correctamente
    [ ] Dashboard: http://localhost:8222

[ ] Añadir NATS_URL a .env.example y .env.local:
    [ ] NATS_URL=nats://localhost:4222
```

**Verificación F2.1:** `docker ps` o `podman ps` muestra nats corriendo.

---

## F2.2 — Crate events + Publisher

> **Referencia:** ADR 0025, ADR 0026, docs/03-STRUCTURE.md L368-380

```
[ ] Crear crates/events/:
    └─ Ref: ADR 0026, docs/03-STRUCTURE.md L368-380

[ ] crates/events/Cargo.toml:
    [ ] [package] name = "events"
    [ ] domain = { path = "../domain" }
    [ ] async-nats = { workspace = true }
        └─ Ref: ADR 0026, docs/02-STACK.md L172-177
    [ ] serde = { workspace = true }
    [ ] thiserror = { workspace = true }
    [ ] tracing = { workspace = true }

[ ] crates/events/src/lib.rs:
    [ ] pub mod publisher;
    [ ] pub mod consumer;
    [ ] pub mod subjects;
    [ ] pub mod error;

[ ] crates/events/src/subjects.rs:
    └─ Ref: ADR 0026 — nomenclatura inamovible
    [ ] pub const USER_REGISTERED: &str = "boilerplate.user.registered.v1";
    [ ] pub const REPORT_REQUESTED: &str = "boilerplate.report.requested.v1";
    [ ] pub const EMAIL_BATCH_QUEUED: &str = "boilerplate.email.batch.v1";
    [ ] Formato: boilerplate.{dominio}.{evento}.{version}

[ ] crates/events/src/publisher.rs:
    └─ Ref: ADR 0026 — publicador genérico
    [ ] pub async fn publish_event<T: Serialize>(js, subject, payload)
    [ ] EventPublisher struct con métodos tipados:
        [ ] user_registered(user_id, email)
        [ ] report_requested(user_id, params)
    [ ] ACK explícito del servidor antes de continuar

[ ] crates/events/src/consumer.rs:
    └─ Ref: ADR 0025 — consumidor durable
    [ ] get_or_create_consumer con durable_name
    [ ] messages().await loop con msg.ack()
    [ ] Sin ACK, NATS reenvía el mensaje (at-least-once)

[ ] crates/events/src/error.rs:
    [ ] #[derive(Error)] EventError
    [ ] Serialization, Publish, Ack, Connection

[ ] Añadir events a workspace root Cargo.toml
    [ ] crates/events en [workspace.members]
```

**Verificación F2.2:** `cargo check -p events` compila sin errores.

---

## F2.3 — Publicar eventos desde el API

> **Referencia:** ADR 0026, ADR 0018 (Apalis)

```
[ ] Inicializar NATS en apps/api:
    [ ] Añadir events = { path = "../../crates/events" } a apps/api/Cargo.toml
    [ ] async-nats = { workspace = true }

[ ] Crear crates/events/src/jetstream.rs:
    └─ Ref: ADR 0025 — setup_jetstream()
    [ ] get_or_create_stream("BOILERPLATE_EVENTS")
    [ ] subjects: vec!["boilerplate.>".to_string()]
    [ ] storage: File, retention: 7 días, max_bytes: 1GB

[ ] Añadir EventsPublisher al estado de Axum:
    [ ] #[derive(Clone)] pub struct AppState { ... events: EventPublisher }
    [ ] Inyectar en handlers que necesiten publicar

[ ] Ejemplo: Publicar evento al crear usuario:
    [ ] En POST /api/v1/users handler:
        [ ] Crear usuario en DB
        [ ] state.events.user_registered(user.id, user.email).await
        [ ] No bloquea la response — ACK es inmediato (~1ms)

[ ] Configurar reintentos en el publicador:
    [ ] Si NATS no disponible, queue local temporal
    [ ] Fallback: usar Apalis como cola de respaldo
```

**Verificación F2.3:** `curl` crea usuario y `nats stream info` muestra mensaje entrante.

---

## F2.4 — Consumidor durable + Worker

> **Referencia:** ADR 0025, ADR 0018

```
[ ] Crear apps/worker/ (binario separado):
    └─ Ref: ADR 0031 Nivel 4 — separación de workers
    [ ] apps/worker/Cargo.toml:
        [ ] [dependencies] reutiliza crates existentes:
            [ ] events, application, database, mailer
        [ ] tokio = { workspace = true }
        [ ] async-nats = { workspace = true }

[ ] apps/worker/src/main.rs:
    [ ] Conectar a NATS desde NATS_URL
    [ ] Crear consumidor durable:
        [ ] durable_name: "worker_email_processor"
        [ ] max_deliver: 3 (reintentos)
    [ ] Loop de mensajes:
        [ ] match msg.subject:
            [ ] boilerplate.user.registered.v1 → enviar email de bienvenida
            [ ] boilerplate.report.requested.v1 → generar PDF
            [ ] boilerplate.email.batch.v1 → procesar batch
        [ ] msg.ack().await? — confirmar procesamiento

[ ] Idempotencia en handlers:
    [ ] Verificar si ya procesado (user_id como clave de deduplicación)
    [ ] Si ya existe, msg.ack() y continuar (sin duplicar trabajo)

[ ] Monitoreo del consumidor:
    [ ] Métricas: mensajes procesados por segundo, lag del consumidor
    [ ] Alerta si lag > 100 mensajes (usando nats consumer info)
```

**Verificación F2.4:** Worker procesa mensajes y ACK confirma.

---

## F2.5 — Separación API vs Worker

> **Referencia:** ADR 0031 (Escalamiento), ADR 0014 (Deploy)

```
[ ] apps/api/ → solo Axum HTTP:
    [ ] Quitar Apalis de apps/api (si estaba)
    [ ] Solo publica eventos a NATS, no procesa jobs pesados
    [ ] Latencia HTTP garantizada <50ms P95

[ ] apps/worker/ → solo procesamiento:
    [ ] Sin endpoints HTTP (solo NATS consumer)
    [ ] Puede consumir 100% CPU sin afectar API
    [ ] Múltiples instancias posibles (scale horizontally)

[ ] Kamal deploy con múltiples roles:
    [ ] servers:
        [ ] web: 1 (API)
        [ ] worker: 1 (puede aumentarse a N workers)
    [ ] Traefik en web, sin Traefik en worker

[ ] Healthcheck diferenciado:
    [ ] API: /health → HTTP 200
    [ ] Worker: NATS consumer info → lag < 100 mensajes
```

**Verificación F2.5:** `kamal deploy` arranca API y worker separados.

---

## Verificaciones de Fase 2

```bash
# 1. NATS está corriendo
docker ps | grep nats  # o podman ps
# Esperado: nats:2.10-alpine Up

# 2. Stream existe
nats stream info BOILERPLATE_EVENTS
# Esperado: Stream: BOILERPLATE_EVENTS, Subjects: boilerplate.>

# 3. Publicar mensaje de prueba
curl -X POST http://localhost:8080/api/v1/users \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"email":"test-events@example.com","password":"12345678"}'
# → Esperado: 201 Created

# 4. Verificar mensaje en stream
nats stream get BOILERPLATE_EVENTS --seq 1
# Esperado: Datos del evento user.registered

# 5. Worker procesó el mensaje
nats consumer info BOILERPLATE_EVENTS worker_email_processor
# Esperado: Num Ack Pending: 0 (todos procesados)

# 6. Latencia API medida
# Hacer 100 requests mientras worker procesa:
for i in {1..100}; do curl -s -o /dev/null -w "%{time_total}\n" http://localhost:8080/health; done
# Esperado: P95 < 0.050s (50ms)

# 7. Sin pérdida de mensajes
# Detener worker, publicar 10 mensajes, arrancar worker
# Esperado: los 10 mensajes son procesados (no se pierden)
```

---

## Diagrama de Flujo Fase 2

```
┌─────────────────────────────────────────────────────────────────┐
│  F2.1 — Infraestructura NATS                                     │
│  ├─ compose.nats.yml con JetStream                              │
│  ├─ Podman/Docker compatible                                    │
│  └─ Verificar: nats stream info                                 │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  F2.2 — Crate events                                             │
│  ├─ crates/events/ con publisher.rs                             │
│  ├─ subjects.rs (constantes inamovibles)                       │
│  ├─ consumer.rs (durable consumer)                              │
│  └─ Verificar: cargo check -p events                            │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  F2.3 — Publicar desde API                                       │
│  ├─ EventPublisher en AppState                                  │
│  ├─ POST /users → publish user.registered                       │
│  ├─ ACK inmediato (~1ms)                                        │
│  └─ Verificar: nats stream get muestra mensajes                 │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  F2.4 — Consumidor durable + Worker                              │
│  ├─ apps/worker/ binario separado                               │
│  ├─ durable_name: worker_email_processor                        │
│  ├─ msg.ack() confirmación explícita                           │
│  ├─ Idempotencia: user_id como clave de dedup                   │
│  └─ Verificar: consumer info → lag bajo                         │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  F2.5 — Separación API vs Worker                                 │
│  ├─ API: solo HTTP, latencia <50ms garantizada                  │
│  ├─ Worker: solo NATS, puede escalar horizontalmente            │
│  ├─ Kamal deploy con roles web + worker                         │
│  └─ Verificar: cero downtime, lag < 100 mensajes                │
└─────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta | URL | Útil para |
|-------------|-----|-----------|
| **NATS Docs** | https://docs.nats.io/ | JetStream, subjects, streams |
| **async-nats** | https://docs.rs/async-nats | SDK Rust |
| **nats-cli** | https://github.com/nats-io/natscli | CLI de diagnóstico |
| **nats-top** | https://github.com/nats-io/nats-top | Monitoreo en tiempo real |

---

## Troubleshooting — Fase 2

### F2.1 — Infraestructura NATS

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `nats: command not found` | CLI no instalado | `go install github.com/nats-io/natscli/nats@latest` — Ref: ADR 0026 |
| `connection refused` | NATS no arrancó | Verificar `docker ps` o `podman ps` — Ref: ADR 0014 |
| Puerto 4222 ocupado | Otro servicio usa el puerto | Cambiar port mapping en compose.nats.yml |
| `JetStream not enabled` | Falta flag `-js` | Verificar command en compose.nats.yml — Ref: ADR 0026 |

### F2.2 — Crate events

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `async_nats not found` | No añadido a workspace | Añadir a [workspace.dependencies] — Ref: docs/02-STACK.md L172-177 |
| `cannot find module publisher` | lib.rs no exporta | Verificar `pub mod publisher;` en lib.rs |
| `subject constant not used` | Imports incorrectos | Usar `use events::subjects::USER_REGISTERED;` |

### F2.3 — Publicar eventos

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `PublishError` timeout | NATS no responde | Verificar NATS_URL y que el servidor esté up |
| Mensajes no aparecen en stream | Stream no existe | Ejecutar setup_jetstream() antes — Ref: ADR 0025 |
| ACK tarda >100ms | NATS en otra máquina | NATS debe estar en mismo VPC que API — Ref: ADR 0026 |

### F2.4 — Consumidor durable

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Mensajes no se procesan | Consumer no creado | Verificar `get_or_create_consumer` — Ref: ADR 0025 |
| Mismo mensaje procesado N veces | Sin ACK | Añadir `msg.ack().await?` — Ref: ADR 0025 |
| `MaxDeliverExceeded` | Handler falla constantemente | Fix error en handler o ajustar max_deliver |
| Lag del consumidor crece | Worker lento | Escala horizontal: más instancias de worker — Ref: ADR 0031 |
| Mensaje procesado pero no ACK | Panic en handler | Usar `msg.double_ack()` para confirmación extra |

### F2.5 — Separación API/Worker

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| API lenta aún con NATS | Worker en mismo host | Separar a máquinas diferentes — Ref: ADR 0031 |
| Worker no arranca | Sin NATS_URL | Configurar variable en .env.local — Ref: ADR 0002 |
| Kamal deploy falla | Rol worker mal definido | Verificar `servers:` en `config/deploy.yml` — Ref: ADR 0014 |

---

## Criterio de Transición a Fase 3

**NO pasar a Fase 3 (KMP + UniFFI) hasta que:**
- Fase 2 completada y estable por >30 días
- >10,000 mensajes/hora sostenidos
- Necesidad de mobile nativo 120Hz medida en producción
- Ver ADR 0031 Nivel 5 y ADR 0030 para criterios específicos

---

**Notas importantes:**
- NATS es el bus más ligero que admite persistencia real (~25MB RAM)
- Los consumidores deben ser idempotentes — usar `user_id` como clave de deduplicación
- El prefijo `boilerplate.*` es inamovible — cambiarlo rompe todos los consumidores
- Ver ADR 0025 para arquitectura de eventos y ADR 0026 para implementación completa
