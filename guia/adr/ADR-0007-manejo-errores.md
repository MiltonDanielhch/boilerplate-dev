# ADR 0007 — Jerarquía de Errores: Domain → App → HTTP

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Dominio puro), ADR 0003 (Axum responses), ADR 0016 (Tracing) |

---

## Contexto

El ADR 0001 define que el dominio no conoce HTTP, pero no establece cómo los errores
del dominio se convierten en respuestas HTTP. Sin una jerarquía clara:

- Cada handler maneja errores de forma diferente — respuestas inconsistentes al cliente
- Los errores de infraestructura (SQLx) se mezclan con errores de negocio
- Los stack traces de producción filtran información sensible si no se controlan

---

## Decisión

Tres niveles de error, cada uno con su responsabilidad clara:

```
DomainError        ← El dominio — reglas de negocio violadas
    ↓ convierte en
AppError           ← La aplicación — errores de casos de uso + infraestructura
    ↓ implementa
IntoResponse       ← HTTP — Axum convierte AppError en respuesta JSON automáticamente
```

### Nivel 1 — DomainError (crates/domain)

Solo errores de reglas de negocio. Sin dependencias externas:

```rust
// crates/domain/src/errors/domain_error.rs
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    // Validación de value objects
    #[error("email inválido: {0}")]
    InvalidEmail(String),

    #[error("contraseña muy corta — mínimo {min} caracteres")]
    PasswordTooShort { min: usize },

    // Reglas de negocio
    #[error("el email ya está registrado")]
    EmailAlreadyExists,

    #[error("credenciales inválidas")]
    InvalidCredentials,

    #[error("token expirado o revocado")]
    InvalidToken,

    #[error("recurso no encontrado: {resource}")]
    NotFound { resource: String },

    #[error("operación no permitida: requiere permiso {permission}")]
    Forbidden { permission: String },

    // Error de infraestructura encapsulado — el mensaje real va al log, no al cliente
    #[error("error de base de datos")]
    Database(String),
}

impl From<sqlx::Error> for DomainError {
    fn from(e: sqlx::Error) -> Self {
        // El error real de SQLx se loggea internamente — el cliente no lo ve
        tracing::error!(error = ?e, "SQLx error");
        Self::Database(e.to_string())
    }
}
```

### Nivel 2 — AppError (crates/domain) + IntoResponse

```rust
// crates/infrastructure/src/errors/app_error.rs
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    // Integración con garde (ADR 0003)
    #[error("error de validación")]
    Validation(#[from] garde::Report),

    #[error("no autenticado")]
    Unauthorized,

    #[error("token expirado")]
    TokenExpired,

    #[error("error interno del servidor")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            // 400 Bad Request
            AppError::Domain(DomainError::InvalidEmail(m))      => (StatusCode::BAD_REQUEST,  "invalid_email",       m.clone()),
            AppError::Domain(DomainError::PasswordTooShort{..}) => (StatusCode::BAD_REQUEST,  "password_too_short",  self.to_string()),

            // 401 Unauthorized
            AppError::Unauthorized | AppError::Domain(DomainError::InvalidToken) =>
                (StatusCode::UNAUTHORIZED, "unauthorized", "No autenticado".into()),
            AppError::TokenExpired =>
                (StatusCode::UNAUTHORIZED, "token_expired", "Token expirado".into()),
            AppError::Domain(DomainError::InvalidCredentials) =>
                (StatusCode::UNAUTHORIZED, "invalid_credentials", "Credenciales inválidas".into()),

            // 403 Forbidden
            AppError::Domain(DomainError::Forbidden{..}) =>
                (StatusCode::FORBIDDEN, "forbidden", self.to_string()),

            // 404 Not Found
            AppError::Domain(DomainError::NotFound{..}) =>
                (StatusCode::NOT_FOUND, "not_found", self.to_string()),

            // 409 Conflict
            AppError::Domain(DomainError::EmailAlreadyExists) =>
                (StatusCode::CONFLICT, "email_already_exists", self.to_string()),

            // 422 Unprocessable Entity (Garde)
            AppError::Validation(report) => {
                let fields: Vec<_> = report.iter().map(|(path, error)| {
                    json!({ "field": path.to_string(), "message": error.to_string() })
                }).collect();
                return (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({ "error": "validation_error", "fields": fields }))).into_response();
            }

            // 500 Internal — nunca exponer detalles
            AppError::Domain(DomainError::Database(_)) | AppError::Internal(_) => {
                tracing::error!(error = ?self, "error interno");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Error interno del servidor".into())
            }
        };

        (status, Json(json!({ "error": code, "message": message }))).into_response()
    }
}
```

### Nivel 3 — Uso en handlers de Axum

```rust
// crates/infrastructure/src/http/handlers/user_handler.rs
pub async fn create_user(
    State(state): State<AppState>,
    Json(body):   Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserDto>), AppError> {
    // DomainError::InvalidEmail → AppError::Domain → 400 automáticamente
    let email = Email::new(&body.email)?;

    // Si el email ya existe → 409 automáticamente
    if state.user_repo.find_active_by_email(&email).await?.is_some() {
        return Err(DomainError::EmailAlreadyExists.into());
    }

    let user = state.create_user_use_case.execute(body.into()).await?;
    Ok((StatusCode::CREATED, Json(user.into())))
}
```

### Formato de respuesta de error — consistente en toda la API

```json
{ "error": "email_already_exists",  "message": "el email ya está registrado" }
{ "error": "invalid_email",         "message": "email inválido: no contiene @" }
{ "error": "forbidden",             "message": "operación no permitida: requiere permiso users:write" }
{ "error": "internal_error",        "message": "Error interno del servidor" }
```

### Errores de validación de formularios (422)

```rust
#[derive(Debug, Serialize)]
pub struct ValidationErrors {
    pub fields: Vec<FieldError>,
}

impl IntoResponse for ValidationErrors {
    fn into_response(self) -> Response {
        (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({
            "error":  "validation_error",
            "fields": self.fields,
        }))).into_response()
    }
}
```

```json
{
  "error": "validation_error",
  "fields": [
    { "field": "email",    "message": "formato inválido" },
    { "field": "password", "message": "mínimo 8 caracteres" }
  ]
}
```

---
| **Curva de aprendizaje** | Requiere entender Traits y Enums | Simple pero propenso a errores |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para mejorar el diagnóstico y la comunicación de errores:

| Herramienta | Propósito en el Manejo de Errores |
| :--- | :--- |
| **`thiserror`** | **Definición de Errores:** La base para crear enums de error con mensajes automáticos. |
| **`miette`** | **Reportes Visuales:** Genera errores legibles y con contexto para desarrollo y CLI. |
| **`tracing-error`** | **Contexto de Spans:** Permite capturar el contexto de la request (ID, User) junto con el error. |
| **`anyhow`** | **Errores de Arranque:** Ideal para el `main.rs` donde solo importa capturar el fallo inicial. |

---

## Consecuencias

### ✅ Positivas

- El dominio solo conoce `DomainError` — sin dependencias de HTTP ni Axum
- Los errores de infraestructura nunca exponen detalles al cliente en producción
- Formato de error consistente — el frontend tiene un solo lugar para manejar errores
- `tracing::error!` en los errores 500 — el stack trace en logs sin llegar al cliente

### ⚠️ Negativas / Trade-offs

- `thiserror` es una dependencia adicional
  → ~0KB en runtime — solo macros de compilación, sin overhead
- El match en `IntoResponse` crece con cada nuevo error
  → Organizar por código HTTP: todos los 400 juntos, todos los 401 juntos, etc.
  → `sintonia g module` (ADR 0028) genera los errores de dominio del módulo automáticamente
  → Un match de 20 casos es más claro que 20 funciones separadas

### Decisiones derivadas

- `thiserror` se agrega como dependencia en `crates/domain`
- Los errores 500 siempre se loggean con `tracing::error!` antes de responder — nunca silenciosos
- El frontend hace `switch(error.error)` para manejar cada caso — el campo `error` es el discriminador
- `sintonia g module` (ADR 0028) genera los errores de dominio del módulo nuevo automáticamente
