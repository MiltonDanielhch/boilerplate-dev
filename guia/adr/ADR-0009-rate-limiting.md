# ADR 0009 — Rate Limiting con tower-governor

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0003 (Axum + Tower middleware), ADR 0008 (Auth — límites por endpoint) |

---

## Contexto

Exponer la API a internet sin límite de peticiones la hace vulnerable a:

- **Fuerza bruta** en endpoints de autenticación (`/auth/login`, `/auth/register`)
- **Scraping** masivo de datos
- **DDoS** accidental o intencional que agote el VPS de $5

Necesitamos límites que protejan el sistema sin afectar a usuarios legítimos.

---

## Decisión

Usar **`tower-governor`** con tres configuraciones: límite estricto para auth, límite
para leads, y límite general para la API.

### Dependencias

```toml
# apps/api/Cargo.toml
tower-governor = { version = "0.4", features = ["axum"] }
```

### Configuraciones por tipo de endpoint

```rust
// apps/api/src/middleware/rate_limit.rs
use tower_governor::{
    governor::GovernorConfigBuilder,
    key_extractor::PeerIpKeyExtractor,
    GovernorLayer,
};

/// Auth — 1 req/s, burst 5: previene fuerza bruta de contraseñas
pub fn auth_rate_limit() -> GovernorLayer<PeerIpKeyExtractor, _> {
    let config = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(5)
        .finish()
        .expect("invalid auth rate limit config");
    GovernorLayer::new(config)
}

/// Leads — 3 req/min: previene spam del formulario de la landing
pub fn leads_rate_limit() -> GovernorLayer<PeerIpKeyExtractor, _> {
    let config = GovernorConfigBuilder::default()
        .per_minute(3)
        .burst_size(3)
        .finish()
        .expect("invalid leads rate limit config");
    GovernorLayer::new(config)
}

/// API general — 10 req/s, burst 30: uso normal de usuario activo
pub fn api_rate_limit() -> GovernorLayer<PeerIpKeyExtractor, _> {
    let config = GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(30)
        .finish()
        .expect("invalid api rate limit config");
    GovernorLayer::new(config)
}
```

### Integración en el router de Axum

```rust
// apps/api/src/router.rs
pub fn build_router(state: AppState) -> Router {
    // Rutas de autenticación — límite estricto anti-fuerza bruta
    let auth_routes = Router::new()
        .route("/auth/login",    post(login_handler))
        .route("/auth/register", post(register_handler))
        .route("/auth/refresh",  post(refresh_handler))
        .layer(auth_rate_limit());

    // Captura de leads — límite estricto anti-spam
    let leads_routes = Router::new()
        .route("/api/v1/leads", post(capture_lead_handler))
        .layer(leads_rate_limit());

    // API general autenticada
    let api_routes = Router::new()
        .route("/api/v1/users",     get(list_users).post(create_user))
        .route("/api/v1/users/:id", get(get_user).put(update_user))
        .layer(auth_middleware)
        .layer(api_rate_limit());

    Router::new()
        .merge(auth_routes)
        .merge(leads_routes)
        .merge(api_routes)
        .merge(docs_router())
        .route("/health", get(health_handler)) // /health excluido del rate limit
        .with_state(state)
}
```

### Respuesta cuando se excede el límite

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
Retry-After: 30

{
  "error":       "too_many_requests",
  "message":     "Rate limit exceeded. Try again in 30 seconds.",
  "retry_after": 30
}
```

### Tabla de límites

| Endpoint | Req/s | Burst | Justificación |
|----------|-------|-------|---------------|
| `/auth/*` (login, register) | 1 | 5 | Prevenir fuerza bruta — argon2id tarda 200ms/intento |
| `/api/v1/leads` | 3/min | 3 | Anti-spam de formularios |
| API general autenticada | 10 | 30 | Uso normal de usuario activo |
| `/health` | Sin límite | — | Necesario para Kamal healthcheck (cada 3s) |

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| Rate limiting en Caddy | Sin contexto de la aplicación — no distingue endpoints con lógica diferente |
| Redis + sliding window | Proceso externo innecesario en MVP — tower-governor es in-process |
| Fail2ban en el VPS | Solo protege a nivel de IP, no a nivel de endpoint |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la precisión y observabilidad del control de tráfico:

| Herramienta | Propósito en el Rate Limiting |
| :--- | :--- |
| **`axum-client-ip`** | **IP Real:** Extrae de forma segura la IP del cliente detrás de proxies (Cloudflare, Caddy, Nginx). |
| **`governor`** | **Motor de Algoritmo:** La base de tower-governor; permite usar algoritmos de "GCRA" (Generic Cell Rate Algorithm). |
| **`metrics`** | **Monitoreo:** Permite contar cuántas veces se activa el 429 para detectar ataques en tiempo real. |
| **`tower-http` (CatchPanic)** | **Resiliencia:** Asegura que un fallo en el rate limit no tire el proceso entero. |

---

## Consecuencias

### ✅ Positivas

- Protección inmediata contra fuerza bruta en auth sin código adicional
- In-process — sin latencia de red adicional ni Redis
- `Retry-After` header automático — los clientes saben cuándo reintentar
- argon2id (ADR 0008) tarda ~200ms/intento — el rate limit de 1 req/s + argon2id = defensa doble

### ⚠️ Negativas / Trade-offs

- El estado del rate limit es in-memory — se resetea en cada reinicio del proceso
  → Aceptable para MVP — Kamal hace deploys zero-downtime, el gap de state es ~0 segundos
  → En Fase 2 con múltiples instancias: evaluar Redis como backend compartido
  → El gap de rate limit tras reinicio no representa riesgo real con argon2id de defensa
- No distingue entre IPs legítimas detrás de un proxy compartido (NAT)
  → Configurar Caddy para pasar `X-Real-IP` — tower-governor usa ese header
  → Si el rate limit es demasiado restrictivo: aumentar el burst_size, no el per_second

### Decisiones derivadas

- El endpoint `/health` está explícitamente excluido — Kamal lo llama cada 3s
- En producción, Caddy pasa el header `X-Real-IP` para que tower-governor use la IP real
- El rate limit de auth es complementario a argon2id — ambos son necesarios
