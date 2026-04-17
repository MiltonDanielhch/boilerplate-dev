// Ubicación: `apps/api/src/docs.rs`
//
// Descripción: Documentación OpenAPI central con Utoipa.
//              Define el spec de la API y endpoints para Scalar UI.
//
// ADRs relacionados: ADR 0021 (OpenAPI)

use utoipa::OpenApi;

/// Documentación OpenAPI central de la API.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Boilerplate API",
        description = "API REST con Rust, Axum, PASETO v4, SQLite y RBAC",
        version = "0.1.0",
        contact(name = "API Support", email = "support@boilerplate.dev"),
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    ),
    servers(
        (url = "http://localhost:3000", description = "Servidor de desarrollo"),
        (url = "https://api.boilerplate.dev", description = "Servidor de producción")
    ),
    paths(
        crate::handlers::health::health,
        crate::handlers::auth::register,
        crate::handlers::auth::login,
        crate::handlers::auth::refresh,
        crate::handlers::auth::logout,
        crate::handlers::users::list,
        crate::handlers::users::get,
        crate::handlers::users::create,
        crate::handlers::users::update,
        crate::handlers::users::soft_delete
    ),
    components(schemas(
        crate::error::ErrorResponse,
        crate::error::ErrorDetail,
        crate::handlers::auth::RegisterRequest,
        crate::handlers::auth::RegisterResponse,
        crate::handlers::auth::LoginRequest,
        crate::handlers::auth::LoginResponse,
        crate::handlers::auth::RefreshRequest,
        crate::handlers::auth::RefreshResponse,
        crate::handlers::users::UserResponse,
        crate::handlers::users::ListUsersResponse,
        crate::handlers::users::CreateUserRequest,
        crate::handlers::users::UpdateUserRequest,
        crate::handlers::users::ListUsersQuery,
        crate::handlers::health::HealthResponse
    )),
    tags(
        (name = "Auth", description = "Autenticación y autorización"),
        (name = "Users", description = "Gestión de usuarios"),
        (name = "Health", description = "Health checks")
    ),
    security(("paseto" = []))
)]
pub struct ApiDoc;
