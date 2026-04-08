# ADR 0016 — Observabilidad: tracing + Sentry + OTLP

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0003 (Axum + TraceLayer), ADR 0008 (PASETO / User ID en spans) |

---

## Contexto

Con un VPS de $5 y 1GB de RAM, un stack de monitoreo local (Grafana + Loki + Prometheus)
consumiría entre 500–700MB — dejando casi nada para la aplicación real.

Necesitamos observabilidad que cumpla con:

- **Impacto mínimo en RAM/CPU** — el monitoreo no compite con la API por recursos
- **Alertas en tiempo real** — saber si el servicio cayó sin entrar al servidor manualmente
- **Costo $0** — aprovechar las capas gratuitas de servicios SaaS modernos

---

## Decisión

Adoptar una estrategia de **telemetría push externa** dividida en tres capas que juntas
usan ~20–40MB de RAM total.

### Capa 1 — Logs estructurados locales (siempre activa)

```toml
# apps/api/Cargo.toml
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
```

```rust
// apps/api/src/telemetry.rs
pub fn init_telemetry(env: &str) {
    let filter = EnvFilter::from_default_env();
    let fmt_layer = tracing_subscriber::fmt::layer().json();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(tracing_sentry::layer())  // Errores críticos a Sentry
        .init();
}
```

### Capa 2 — Reporte de errores (Sentry SDK)

Captura panics y errores críticos automáticamente:

```toml
# apps/api/Cargo.toml
sentry         = "0.34"
tracing-sentry = "0.10"
```

```rust
// apps/api/src/telemetry.rs
let _guard = sentry::init((
    std::env::var("SENTRY_DSN").unwrap_or_default(),
    sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    },
));
```

### Capa 3 — Trazas e insights (OTLP → Axiom)

Solo en producción — no satura el tráfico local en desarrollo:

```rust
// apps/api/src/telemetry.rs
if env == "production" {
    let exporter = opentelemetry_otlp::new_exporter()
        .http()  // HTTP es más ligero que gRPC para un VPS pequeño
        .with_endpoint(
            std::env::var("OTLP_ENDPOINT")
                .unwrap_or("https://otlp.axiom.co/v1/traces".into())
        );
    // ... configuración del tracer provider
}
```

### Estrategia de correlación sin RAM extra

| Nivel | Herramienta | Qué guarda | Límite |
|-------|-------------|-----------|--------|
| Local | `tracing` + JSON | Todos los logs | 10MB rotación Podman |
| Remoto | Sentry | Solo errores y panics — con `request_id` | Capa gratuita |
| Remoto | Axiom OTLP | Solo trazas lentas (>100ms) — no todo | Capa gratuita |

El `request_id` generado en ADR 0003 es el hilo que conecta el log local con el error en la nube.

### Debugging en desarrollo local

```bash
# Ver logs en tiempo real con formato legible
RUST_LOG=debug,sqlx=warn,tower_http=debug just dev-api 2>&1 | jq .

# Para testear que Sentry recibe errores antes de producción:
SENTRY_DSN=tu_dsn cargo run
# Verificar en el dashboard de Sentry que llegó el evento

# Para ver trazas locales sin Axiom (Jaeger en Docker):
docker run -p 16686:16686 -p 4318:4318 jaegertracing/all-in-one
OTLP_ENDPOINT=http://localhost:4318 just dev-api
# Abrir http://localhost:16686
```

---

## Comparativa de carga en VPS ($5)

| Componente | Stack local (Loki + Grafana) | Stack externo (propuesto) |
|------------|------------------------------|--------------------------|
| **RAM** | 500–700MB | 20–40MB |
| **Disco** | Alto — bases de datos de logs | Mínimo — rotación Podman 10MB |
| **Alertas** | Configuración manual compleja | Nativas (Email / Slack / Push) |
| **Costo** | $0 pero mata el VPS | $0 en capa gratuita |

---

## Fases de evolución

### Fase 1 — MVP (actual)
`tracing` JSON + Sentry + Axiom OTLP en producción. Suficiente para producción temprana.

### Fase 2 — Crecimiento
```toml
tracing-opentelemetry = "0.24"
opentelemetry         = "0.23"
opentelemetry-otlp    = "0.16"
```
Métricas de latencia por endpoint (histogramas) + traces distribuidos.

### Fase 3 — Escala real
Loki (logs) + Tempo (traces) + Grafana (dashboards). Solo cuando el VPS suba a $20+.

---

2. Herramientas y Librerías para Optimizar (Edición 2026)
Para que esta estrategia sea aún más potente y fácil de implementar, te sugiero estas herramientas y crates:

tracing-opentelemetry: Es el puente que conecta tu sistema de tracing con el mundo de OpenTelemetry. Permite que tus spans de tracing se conviertan automáticamente en spans de OTLP.
opentelemetry-sdk: El SDK central de OpenTelemetry. Aunque ya lo mencionas en Fase 2, si estás enviando a OTLP en Fase 1, necesitas el SDK para configurar el exportador.
sentry-tower: Para integrar Sentry directamente en tu stack de Tower/Axum. Captura automáticamente el contexto de la petición HTTP (headers, IP, etc.) cuando ocurre un error.
metrics (crate): Aunque ya lo mencionamos en ADR-0003, es importante recalcar su uso aquí para instrumentar tu código con contadores y histogramas que luego pueden ser exportados vía OpenTelemetry.
tracing-subscriber (con fmt y json features): Ya lo usas, pero es clave para tener una salida dual: logs JSON para máquinas en producción y logs bonitos y legibles para humanos en desarrollo.
tracing-error: Ya lo incluimos en ADR-0007, pero es fundamental aquí para adjuntar el contexto de error a los spans de tracing, lo que mejora la depuración.
---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| Grafana + Loki + Prometheus self-hosted | 500–700MB RAM — inviable en VPS de $5 |
| GlitchTip self-hosted | Requiere su propia DB y proceso — overhead en MVP |
| Solo logs locales | Sin alertas — hay que entrar al servidor para detectar problemas |

---

## Consecuencias

### ✅ Positivas

- La API de Rust tiene el 90% de la RAM disponible para procesar requests
- Alertas automáticas por email sin configuración adicional
- Los logs detallados se quedan en el VPS; solo errores y trazas viajan fuera
- Si el proyecto crece, solo se cambia el plan del SaaS — el servidor no cambia

### ⚠️ Negativas / Trade-offs

- Dependencia de conexión a internet — si el servidor pierde red, los errores no se reportan
  → Los logs locales en JSON siguen funcionando sin internet — disponibles con `podman logs`
  → El buffer de Sentry reenvía los errores cuando vuelve la conexión
- Stack traces viven en servidores externos (Sentry/Axiom), aunque cifrados
  → Para proyectos con datos muy sensibles: GlitchTip self-hosted cuando el VPS suba a $10+
- Visibilidad limitada en desarrollo local sin el dashboard del SaaS
  → Ver sección "Debugging en desarrollo local" arriba — Jaeger local resuelve esto

### Decisiones derivadas

- El `request_id` de ADR 0003 es el campo obligatorio en todos los eventos de telemetría
- El driver de logs de Podman se configura con límite de 10MB para no agotar el disco
- Se usa HTTP en lugar de gRPC para el envío de telemetría — ahorra ciclos de CPU
- `SENTRY_DSN` y `OTLP_ENDPOINT` son opcionales en `.env.example` — el sistema arranca sin ellos
