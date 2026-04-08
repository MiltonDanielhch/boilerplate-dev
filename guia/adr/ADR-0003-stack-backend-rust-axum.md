# ADR 0003 — Stack Backend: Rust 2024 + Axum 0.8 + Tokio

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Revisado en** | ADR 0014 (Deploy distroless) |

---

## Contexto

Necesitamos un servidor web para el boilerplate que cumpla con estos requisitos no negociables:

- **Alto rendimiento** con latencias por debajo de los 10ms en endpoints típicos
- **Manejo asíncrono eficiente** sin el overhead de un thread por request
- **Seguridad de memoria** sin garbage collector ni condiciones de carrera
- **Ecosistema maduro** de crates para auth, DB, observabilidad y middleware
- **Binario único** que se pueda distribuir en una imagen distroless de ~10MB

La alternativa evaluada fue Go (net/http + Fiber), pero la seguridad de tipos de Rust y la
garantía en tiempo de compilación de ausencia de data races fue determinante para sistemas
que manejan datos de usuario.

---

## Decisión

Usar **Rust (Edition 2024)** con **Axum 0.8** y **Tokio** como runtime asíncrono.

### Stack completo del servidor

```toml
# apps/api/Cargo.toml
axum          = { version = "0.8", features = ["macros"] }
axum-extra    = { version = "0.9", features = ["cookie", "typed-header"] }
tower         = "0.5"
tower-http    = { version = "0.6", features = [
    "cors", "compression-gzip", "compression-br",
    "trace", "timeout", "request-id"
] }
tower-governor = "0.4"   # Rate limiting — ADR 0009
tokio          = { version = "1", features = ["full"] }
```

### Middleware configurado en orden

El orden del middleware importa — cada capa envuelve a las siguientes:

```rust
// apps/api/src/main.rs
let app = Router::new()
    .merge(api_router())
    .with_state(state)
    .layer(
        ServiceBuilder::new()
            // 1. request_id primero — disponible para todos los layers siguientes
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            // 2. Tracing usa el request_id del layer anterior
            .layer(TraceLayer::new_for_http()
                .make_span_with(|req: &Request<_>| {
                    let request_id = req
                        .headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown");
                    tracing::info_span!("http_request",
                        request_id = %request_id,
                        method     = %req.method(),
                        uri        = %req.uri(),
                    )
                })
            )
            // 3. Compresión de la respuesta
            .layer(CompressionLayer::new())
            // 4. CORS configurado por entorno
            .layer(CorsLayer::new()
                .allow_origin(cors_origin)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([AUTHORIZATION, CONTENT_TYPE])
            )
            // 5. Timeout global — ningún request puede tardar más de 30s
            .layer(TimeoutLayer::new(Duration::from_secs(30)))
            // 6. Rate limiting — ADR 0009
            // (tower-governor se configura por ruta, no en el stack global)
    );
```

### Graceful shutdown

Espera señales SIGTERM (Kamal/systemd) y SIGINT (Ctrl+C) antes de cerrar:

```rust
// apps/api/src/main.rs
let shutdown = async {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    tokio::select! {
        _ = ctrl_c    => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received — draining connections");
};

axum::serve(listener, app)
    .with_graceful_shutdown(shutdown)
    .await?;
```

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|-------------------|
| Go + Fiber | Sin garantías de memoria en tiempo de compilación |
| Node.js + Fastify | Single-threaded, mayor consumo de RAM |
| Java + Spring Boot | JVM overhead, imagen mínima >100MB |
| Python + FastAPI | Performance insuficiente para objetivos del lab |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la productividad y observabilidad del stack de Axum:

| Herramienta | Propósito en el Stack |
| :--- | :--- |
| **`utoipa`** | **Autodocumentación:** Genera OpenAPI/Swagger directamente desde el código para consumo de humanos y agentes IA. |
| **`axum-test`** | **Testing ultra-rápido:** Permite testear el router completo en memoria sin levantar interfaces de red. |
| **`tower-sessions`** | **Gestión de Sesiones:** El estándar para manejar estados de usuario de forma segura y modular. |
| **`tokio-console`** | **Debug Asíncrono:** Una herramienta tipo "top" para tareas de Tokio; permite ver qué requests están bloqueadas. |
| **`garde`** | **Validación Type-safe:** La librería más moderna y rápida para asegurar que los datos de entrada son correctos. |
| **`metrics`** | **Telemetría:** Captura de histogramas y contadores de performance con impacto casi nulo en latencia. |

---

## Consecuencias

### ✅ Positivas

- Seguridad de memoria y concurrencia garantizada por el compilador
- Cero data races — detectadas en compile-time, no en runtime
- Binario estático final ~7–10MB con imagen distroless
- Throughput comparable a C++ con código de alto nivel
- Middleware composable con Tower — cada capa es testeable de forma aislada

### ⚠️ Negativas / Trade-offs

- Curva de aprendizaje inicial alta (ownership, lifetimes, async)
  → Mitigación: la arquitectura hexagonal del ADR 0001 aísla la complejidad de Rust
    en los adaptadores — el dominio es Rust sencillo sin lifetimes complejos
  → Mitigación: `sccache` para cachear compilaciones entre sesiones;
    `mold` como linker en Linux reduce el link time ~50%;
    en CI cachear `~/.cargo/registry` y `target/` entre runs
  → Los cambios incrementales con `cargo-watch` tardan ~5 segundos
- Tiempos de compilación más largos que Go o Node
  → Primera compilación completa ~3 minutos; compilaciones incrementales ~5 segundos
  → `cargo-nextest` paraleliza tests — 3-5x más rápido que `cargo test`
- Ecosistema más joven que Java/Go para algunas integraciones empresariales
  → Para integraciones con sistemas legados (SOAP, EDI): usar `reqwest` hacia
    un microservicio adaptador — el dominio en Rust no cambia
  → El dominio es independiente — la integración vive en `crates/infrastructure`
    y puede ser reemplazada sin tocar la lógica de negocio

### Decisiones derivadas

- El pool de SQLite se configura con máximo 10 conexiones → ver **ADR 0004**
- La autenticación usa middleware de Tower → ver **ADR 0008**
- La estrategia de rate limiting con tower-governor → ver **ADR 0009**
- El deploy usa imagen distroless para aprovechar el binario estático → ver **ADR 0014**
