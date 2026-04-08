# ADR 0021 — OpenAPI: Utoipa + Scalar + IA-Ready

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0003 (Axum), ADR 0027 (ConnectRPC — contratos de API) |

---

## Contexto

La documentación de API que no se actualiza sola es documentación que miente. Necesitamos
que la especificación OpenAPI sea generada directamente desde el código de Rust — sin archivos
YAML escritos a mano que se desincronicen con la implementación real.

Requisitos:

- **Sincronizada** — cambia automáticamente cuando cambia el código
- **Interactiva** — permite probar endpoints directamente desde el browser
- **IA-ready** — el `openapi.json` puede ser consumido por agentes para entender la API
- **Ligera** — sin penalizar el VPS de $5

---

## Decisión

Usar **Utoipa** para generar el esquema OpenAPI mediante macros de Rust, y **Scalar** como
interfaz visual — más moderna y eficiente que Swagger UI.

### Dependencias

```toml
# apps/api/Cargo.toml
utoipa        = { version = "4", features = ["axum_extras", "uuid"] }
utoipa-scalar = { version = "0.1", features = ["axum"] }
```

### Anotación de handlers y DTOs

Las macros solo van en la capa de infraestructura — nunca en entidades del dominio (ADR 0001):

```rust
// crates/infrastructure/src/http/handlers/user_handler.rs
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id:         String,
    pub email:      String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    #[schema(example = "user@example.com")]
    pub email:    String,
    #[schema(example = "password_seguro_123", min_length = 8)]
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "Usuario creado",  body = UserDto),
        (status = 409, description = "Email duplicado", body = ErrorResponse),
        (status = 422, description = "Datos inválidos", body = ValidationError),
    ),
    security(("bearer_auth" = [])),
    tag = "users",
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(body):   Json<CreateUserRequest>,
) -> impl IntoResponse {
    // ...
}
```

### Registro central del spec OpenAPI

```rust
// apps/api/src/docs.rs
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

#[derive(OpenApi)]
#[openapi(
    paths(
        user_handler::create_user,
        user_handler::get_user,
        user_handler::list_users,
        auth_handler::login,
        auth_handler::logout,
        auth_handler::refresh,
        leads_handler::capture_lead,
    ),
    components(schemas(
        UserDto, CreateUserRequest,
        LoginRequest, AuthResponse,
        ErrorResponse, ValidationError,
    )),
    security(("bearer_auth" = [])),
    modifiers(&SecurityAddon),
    info(
        title = "Boilerplate API",
        version = env!("CARGO_PKG_VERSION"),
        description = "API del proyecto boilerplate — Laboratorio 3030",
    )
)]
pub struct ApiDoc;

struct SecurityAddon;
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("PASETO")  // No JWT — PASETO v4 Local
                        .build(),
                ),
            );
        }
    }
}

pub fn docs_router() -> Router {
    Router::new()
        .route("/openapi.json", get(|| async { Json(ApiDoc::openapi()) }))
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
}
```

### Integración en el router principal

```rust
// apps/api/src/router.rs
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", api_router(state))
        .merge(docs_router())          // /docs y /openapi.json
        .route("/health", get(health_handler))
}
```

Con esto, la documentación está disponible en:

- `/docs` — interfaz visual Scalar para explorar y testear
- `/openapi.json` — spec descargable para herramientas externas e IAs

---

## Comparativa: Scalar vs Swagger UI

| Característica | Swagger UI | Scalar |
|----------------|-----------|--------|
| **Diseño** | Anticuado (estilo 2014) | Moderno y limpio |
| **Rendimiento** | Lento en specs grandes | Ultra ligero |
| **Búsqueda** | Básica | Instantánea y por modelo |
| **Autenticación** | Manual cada sesión | Persistente en el browser |
| **Tamaño del bundle** | ~1MB | ~100KB |

---
Herramientas y Librerías para Optimizar (Edición 2026)
He notado que la sección "Herramientas y Librerías para Optimizar" en tu ADR-0021 contenía herramientas relacionadas con el almacenamiento (ADR-0020). He corregido esto para que refleje las herramientas específicas de OpenAPI.

Para que tu sistema de documentación sea aún más potente y útil, te sugiero estas adiciones:

utoipa-redoc: Si bien Scalar es excelente, Redoc ofrece otra interfaz de usuario moderna y de una sola página que algunos desarrolladores prefieren. Tener la opción de ofrecer ambas puede ser un plus.
openapi-typescript-codegen (o similar): Esta herramienta (o una equivalente en el ecosistema JS/TS) puede consumir tu openapi.json y generar automáticamente clientes de API type-safe para tu frontend (Astro/Svelte). Esto elimina la necesidad de escribir manualmente los tipos y las llamadas a la API, garantizando la coherencia entre frontend y backend.
spectral: Un linter para especificaciones OpenAPI. Puedes integrarlo en tu CI (ADR-0010) para validar que el openapi.json generado no solo es válido, sino que también cumple con las mejores prácticas y estándares de tu organización.
cargo-udeps: Aunque es una herramienta general, es útil aquí para asegurar que las dependencias de utoipa y sus features solo se incluyan en apps/api/Cargo.toml y no se filtren a otros crates donde no son necesarias, manteniendo la ligereza del binario.

---

## Consecuencias

### ✅ Positivas

- El código Rust es la única fuente de verdad — sin YAML desincronizado
- IA-ready: `openapi.json` permite a agentes como Windsurf entender la API al instante
- Probar flujos de auth directamente desde `/docs` — sin Postman para desarrollo básico
- El esquema de seguridad muestra "PASETO" como formato del Bearer — documenta la decisión de ADR 0008
- Utoipa verifica en compilación que los tipos referenciados existen

### ⚠️ Negativas / Trade-offs

- Las macros `#[utoipa::path]` añaden verbosidad a los handlers
  → Compensado por la exactitud — si el tipo no existe, el compilador falla
  → `sintonia g module` genera la anotación automáticamente para módulos nuevos (ADR 0028)
- Cada nuevo handler debe registrarse manualmente en el struct `ApiDoc`
  → Fácil de olvidar — pero el spec simplemente omite el endpoint, no falla
  → `sintonia g module` (ADR 0028) registra automáticamente

### Decisiones derivadas

- `/docs` solo está disponible en entornos `development` y `staging` — nunca en producción directa
- `/openapi.json` sí está disponible en producción — para IAs y herramientas de integración
- Las macros de Utoipa van únicamente en DTOs y handlers de infraestructura — nunca en entidades del dominio
- El `bearer_format` en `SecurityAddon` es `"PASETO"` — no `"JWT"` — documenta la decisión del ADR 0008
