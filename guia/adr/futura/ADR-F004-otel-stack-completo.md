# ADR F-004 — Futuro: Stack OTel Completo (Loki + Tempo + Grafana)

| Campo | Valor |
|-------|-------|
| **Estado** | 🔮 Futuro — activar en ADR 0031 Nivel 3+ |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0016 (Observabilidad), ADR 0031 (Escalamiento), ADR 0015 (Healthchecks) |

---

## Contexto

Este ADR documenta la migración a un **stack completo de observabilidad** con **Loki** (logs),
**Tempo** (traces), y **Grafana** (dashboards) cuando el sistema actual (tracing + Sentry + Axiom)
ya no sea suficiente para la escala operativa.

**Activar cuando:**
- El VPS ha crecido a $40+/mes (4+ vCPU, 8+ GB RAM)
- Se necesitan dashboards personalizados para múltiples equipos
- Retención de logs >30 días con querying eficiente
- Correlación log ↔ trace ↔ métrica en una sola interfaz
- Alerting avanzado basado en patrones de logs

**NO activar en MVP o Fase 2.** El stack actual (tracing JSON + Sentry) es suficiente hasta
~100.000 usuarios activos.

---

## Decisión futura

Implementar el stack **Grafana Labs**: Loki + Tempo + Grafana + Alertmanager.

### Arquitectura del stack

```
┌────────────────────────────────────────────────────────────────┐
│                           Grafana                               │
│         (Dashboards unificados: logs, traces, métricas)          │
└────────────────────────┬───────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│    Loki      │ │    Tempo     │ │  Prometheus  │
│  (Logs)      │ │  (Traces)    │ │ (Métricas)   │
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │                │                │
       ▼                ▼                ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│  S3/Tigris   │ │  S3/Tigris   │ │  TSDB local  │
│  (storage)   │ │  (storage)   │ │  (retention) │
└──────────────┘ └──────────────┘ └──────────────┘
```

### Flujo de telemetría

```
apps/api/
    │
    ├──► tracing-subscriber (JSON) ──► Loki (logs)
    │
    ├──► opentelemetry-otlp ──► Tempo (traces)
    │
    └──► metrics (prometheus) ──► Prometheus ──► Grafana (métricas)
```

---

## Cuándo activar

| Criterio | Umbral |
|----------|--------|
| VPS | >= 4 vCPU / 8 GB RAM ($40+/mes) |
| Logs | >1GB/día de logs generados |
| Retención | Necesidad de retención >30 días |
| Equipos | >3 developers necesitan acceso a logs simultáneo |
| Alerting | Necesidad de alertas basadas en patrones de logs (no solo métricas) |
| Correlación | Necesidad de correlacionar un error con su trace_id y sus logs en un clic |

---

## Implementación

### 1. Infraestructura Docker Compose

```yaml
# infra/docker/compose.observability.yml
services:
  grafana:
    image: grafana/grafana:10.3.0
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD}
      - GF_INSTALL_PLUGINS=grafana-lokiexplore-app,grafana-tempoexplore-app
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
    restart: unless-stopped

  loki:
    image: grafana/loki:2.9.0
    ports:
      - "3100:3100"
    volumes:
      - ./loki/loki-config.yml:/etc/loki/local-config.yaml
      - loki_data:/loki
    command: -config.file=/etc/loki/local-config.yaml
    restart: unless-stopped

  tempo:
    image: grafana/tempo:2.3.0
    ports:
      - "3200:3200"  # Tempo
      - "4317:4317"  # OTLP gRPC
      - "4318:4318"  # OTLP HTTP
    volumes:
      - ./tempo/tempo-config.yml:/etc/tempo.yaml
      - tempo_data:/var/tempo
    command: -config.file=/etc/tempo.yaml
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:v2.48.0
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=30d'
    restart: unless-stopped

  alertmanager:
    image: prom/alertmanager:v0.26.0
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager/alertmanager.yml:/etc/alertmanager/alertmanager.yml
    restart: unless-stopped

volumes:
  grafana_data:
  loki_data:
  tempo_data:
  prometheus_data:
```

### 2. Configuración Loki

```yaml
# infra/docker/loki/loki-config.yml
auth_enabled: false

server:
  http_listen_port: 3100
  grpc_listen_port: 9096

common:
  path_prefix: /loki
  storage:
    filesystem:
      chunks_directory: /loki/chunks
      rules_directory: /loki/rules
  replication_factor: 1
  ring:
    instance_addr: 127.0.0.1
    kvstore:
      store: inmemory

schema_config:
  configs:
    - from: 2020-10-24
      store: boltdb-shipper
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb_shipper:
    active_index_directory: /loki/boltdb-shipper
    cache_location: /loki/boltdb-shipper-cache
    cache_ttl: 24h

limits_config:
  retention_period: 720h  # 30 días
  reject_old_samples: true
  reject_old_samples_max_age: 168h
```

### 3. Configuración Tempo

```yaml
# infra/docker/tempo/tempo-config.yml
server:
  http_listen_port: 3200

 distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: 0.0.0.0:4317
        http:
          endpoint: 0.0.0.0:4318

ingester:
  max_block_duration: 5m

compactor:
  compaction:
    block_retention: 720h  # 30 días

storage:
  trace:
    backend: local
    local:
      path: /var/tempo/traces
    wal:
      path: /var/tempo/wal
```

### 4. Configuración Prometheus

```yaml
# infra/docker/prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

rule_files:
  - /etc/prometheus/rules/*.yml

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'boilerplate-api'
    static_configs:
      - targets: ['api:8080']
    metrics_path: /metrics
    scrape_interval: 5s
```

### 5. Configuración Rust (OpenTelemetry)

```toml
# Cargo.toml — crates/observability
[dependencies]
opentelemetry = "0.23"
opentelemetry-otlp = { version = "0.16", features = ["grpc-tonic"] }
opentelemetry-stdout = "0.5"  # Para desarrollo
opentelemetry-semantic-conventions = "0.15"
tracing-opentelemetry = "0.24"
```

```rust
// crates/observability/src/tracing_otlp.rs
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{Sampler, TracerProvider as SdkTracerProvider};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

pub fn init_otlp_tracing(service_name: &str, tempo_url: &str) {
    // Exportador OTLP a Tempo
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(tempo_url);

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter, tokio::runtime::Handle::current())
        .with_sampler(Sampler::AlwaysOn)
        .with_resource(opentelemetry_sdk::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", service_name.to_string()),
        ]))
        .build();

    opentelemetry::global::set_tracer_provider(tracer_provider);

    // Layer de tracing para OpenTelemetry
    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(opentelemetry::global::tracer(service_name));

    // Susbcriptor combinado: stdout + OTLP
    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
```

### 6. Envío de logs a Loki

```rust
// Usar tracing-loki o promtail sidecar
// Opción A: Directo con tracing-loki
use tracing_loki::LokiLayer;

let (loki_layer, task) = LokiLayer::new(
    loki_url,
    vec![("service", "boilerplate-api")].into_iter().collect(),
    vec![].into_iter().collect(),
)?;

tracing_subscriber::registry()
    .with(loki_layer)
    .init();

// Opción B: Promtail sidecar (recomendado para producción)
// Promtail lee los logs JSON de stdout y los envía a Loki
```

---

## Dashboards de Grafana

### 1. Dashboard de API Health

```json
{
  "dashboard": {
    "title": "API Health",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{endpoint}}"
          }
        ]
      },
      {
        "title": "Latency P95",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "P95"
          }
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "rate(http_requests_total{status=~\"5..\"}[5m])",
            "legendFormat": "Errors"
          }
        ]
      }
    ]
  }
}
```

### 2. Dashboard de Logs (Loki)

- Query: `{service="boilerplate-api"} |= "ERROR"`
- Time range: Last 1 hour
- Panel: Logs list with filtering

### 3. Dashboard de Traces (Tempo)

- Query: `trace_id = "abc123"`
- Panel: Flame graph de spans
- Correlación con logs del mismo trace_id

---

## Alerting avanzado

```yaml
# infra/docker/prometheus/rules/api.yml
groups:
  - name: api_alerts
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors per second"

      - alert: HighLatencyP95
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency detected"
          description: "P95 latency is {{ $value }}s"

      - alert: DatabaseConnectionErrors
        expr: rate(database_connection_errors_total[5m]) > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Database connection errors"
```

---

## Consecuencias

### ✅ Positivas

- **Observabilidad unificada:** logs, traces, métricas en un solo lugar (Grafana)
- **Correlación:** clic en un error → ver el trace completo → ver los logs del span
- **Retención:** 30+ días de datos históricos con querying eficiente
- **Costo:** Loki y Tempo son más eficientes que Elasticsearch/Jaeger para el mismo volumen
- **Comunidad:** Stack estándar de Grafana Labs con documentación extensa

### ⚠️ Negativas / Trade-offs

- **Recursos:** Requiere VPS $40+/mes solo para el stack de observabilidad
- **Complejidad:** 4 servicios adicionales (Grafana, Loki, Tempo, Prometheus) vs 1 (Sentry)
- **Setup:** Configuración inicial más compleja que Sentry (solo DSN)
- **Mantenimiento:** Backups de Grafana dashboards, upgrades de versión, etc.
- **Monitoreo del monitoreo:** ¿Quién monitorea que Grafana está up? (Healthchecks.io)

### Decisiones derivadas

- Mantener Sentry para errores críticos (alerting por email/Slack) incluso con Loki
- Loki es complementario, no reemplazo de Sentry para el MVP
- Usar promtail sidecar en lugar de envío directo desde Rust (más robusto)
- Retención de 30 días por defecto — ajustar según presupuesto de storage

---

## Estado actual

**No implementar.** Mantener el stack actual (ADR 0016) hasta que:
1. VPS esté en plan $40+/mes con recursos sobrantes
2. Se necesite retención >30 días de logs
3. Múltiples equipos necesiten acceso simultáneo a logs
4. Sentry ya no sea suficiente para el volumen de errores

**Transición gradual:**
1. Fase 1-2: Sentry + Axiom OTLP (actual)
2. Fase 3: Añadir Loki para logs de larga retención
3. Fase 4+: Añadir Tempo para tracing distribuido completo

Ver ADR 0016 para el stack actual y ADR 0031 para criterios de escalamiento.
