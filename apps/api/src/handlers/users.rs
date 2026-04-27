// Ubicación: `apps/api/src/handlers/users.rs`
//
// Descripción: Handlers CRUD para usuarios.
//
// ADRs relacionados: ADR 0003 (Axum), ADR 0006 (Soft Delete)

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use application::users::{GetUserUseCase, ListUsersUseCase, SoftDeleteUserUseCase, UpdateUserUseCase, ImpersonateUserUseCase};
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use domain::value_objects::UserId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// GET /api/v1/users/:id
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = String, Path, description = "ID del usuario")
    ),
    security(
        ("paseto" = [])
    ),
    responses(
        (status = 200, description = "Usuario encontrado", body = UserResponse),
        (status = 401, description = "No autenticado", body = crate::error::ErrorResponse),
        (status = 403, description = "Sin permiso", body = crate::error::ErrorResponse),
        (status = 404, description = "Usuario no encontrado", body = crate::error::ErrorResponse),
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<UserResponse>> {
    let use_case = GetUserUseCase::new(state.user_repo);
    let id = UserId::parse(&id).map_err(|_| ApiError::Validation("Invalid user ID".to_string()))?;
    
    let user = use_case.execute(&id).await?;
    
    Ok(Json(UserResponse::from(user)))
}

/// GET /api/v1/users
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    params(
        ("limit" = Option<i64>, Query, description = "Cantidad máxima de usuarios"),
        ("offset" = Option<i64>, Query, description = "Offset para paginación"),
    ),
    security(
        ("paseto" = [])
    ),
    responses(
        (status = 200, description = "Lista de usuarios", body = ListUsersResponse),
        (status = 401, description = "No autenticado", body = crate::error::ErrorResponse),
        (status = 403, description = "Sin permiso", body = crate::error::ErrorResponse),
    )
)]
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> ApiResult<Json<ListUsersResponse>> {
    let use_case = ListUsersUseCase::new(state.user_repo);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let input = application::users::ListUsersInput { 
        limit, 
        offset,
        search: params.search,
        role: params.role,
        is_active: params.is_active,
    };
    
    let users = use_case.execute(input).await?;
    let total = users.len() as i64; // TODO: Implementar conteo real en DB
    
    Ok(Json(ListUsersResponse {
        users: users.into_iter().map(UserResponse::from).collect(),
        total,
        limit,
        offset,
    }))
}

/// POST /api/v1/users
#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserRequest,
    security(
        ("paseto" = [])
    ),
    responses(
        (status = 201, description = "Usuario creado", body = UserResponse),
        (status = 400, description = "Datos inválidos", body = crate::error::ErrorResponse),
        (status = 401, description = "No autenticado", body = crate::error::ErrorResponse),
        (status = 403, description = "Sin permiso", body = crate::error::ErrorResponse),
        (status = 409, description = "Email ya existe", body = crate::error::ErrorResponse),
    )
)]
pub async fn create(
    State(_state): State<AppState>,
    Json(_body): Json<CreateUserRequest>,
) -> ApiResult<Json<UserResponse>> {
    // TODO: Implementar cuando tengamos el caso de uso completo
    Err(ApiError::Internal("Not implemented".to_string()))
}

/// PUT /api/v1/users/:id
#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = String, Path, description = "ID del usuario")
    ),
    request_body = UpdateUserRequest,
    security(
        ("paseto" = [])
    ),
    responses(
        (status = 200, description = "Usuario actualizado", body = UserResponse),
        (status = 400, description = "Datos inválidos", body = crate::error::ErrorResponse),
        (status = 401, description = "No autenticado", body = crate::error::ErrorResponse),
        (status = 403, description = "Sin permiso", body = crate::error::ErrorResponse),
        (status = 404, description = "Usuario no encontrado", body = crate::error::ErrorResponse),
    )
)]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateUserRequest>,
) -> ApiResult<Json<UserResponse>> {
    let use_case = UpdateUserUseCase::new(state.user_repo);
    let id = UserId::parse(&id).map_err(|_| ApiError::Validation("Invalid user ID".to_string()))?;
    let input = application::users::UpdateUserInput { 
        name: body.name,
        is_active: body.is_active,
        role: body.role,
    };
    
    let user = use_case.execute(&id, input).await?;
    
    Ok(Json(UserResponse::from(user)))
}

/// POST /api/v1/users/:id/impersonate
#[utoipa::path(
    post,
    path = "/api/v1/users/{id}/impersonate",
    tag = "Users",
    params(
        ("id" = String, Path, description = "ID del usuario objetivo")
    ),
    security(
        ("paseto" = [])
    ),
    responses(
        (status = 200, description = "Token generado", body = ImpersonateResponse),
        (status = 401, description = "No autenticado"),
        (status = 403, description = "Sin permiso (admin only)"),
    )
)]
pub async fn impersonate(
    State(state): State<AppState>,
    claims: crate::middleware::auth::AuthClaims,
    Path(id): Path<String>,
) -> ApiResult<Json<ImpersonateResponse>> {
    let use_case = ImpersonateUserUseCase::new(state.user_repo, state.paseto);
    
    let admin_id = UserId::parse(&claims.user_id)
        .map_err(|_| ApiError::Internal("Invalid admin_id in token".to_string()))?;
    let target_user_id = UserId::parse(&id)
        .map_err(|_| ApiError::Validation("Invalid target user ID".to_string()))?;

    let token = use_case.execute(application::users::ImpersonateUserInput {
        target_user_id,
        admin_id,
    }).await?;

    Ok(Json(ImpersonateResponse { access_token: token }))
}

/// DELETE /api/v1/users/:id — Soft delete (ADR 0006)
#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "Users",
    params(
        ("id" = String, Path, description = "ID del usuario")
    ),
    security(
        ("paseto" = [])
    ),
    responses(
        (status = 200, description = "Usuario eliminado (soft delete)"),
        (status = 401, description = "No autenticado", body = crate::error::ErrorResponse),
        (status = 403, description = "Sin permiso", body = crate::error::ErrorResponse),
        (status = 404, description = "Usuario no encontrado", body = crate::error::ErrorResponse),
    )
)]
pub async fn soft_delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<()>> {
    let use_case = SoftDeleteUserUseCase::new(state.user_repo);
    let id = UserId::parse(&id).map_err(|_| ApiError::Validation("Invalid user ID".to_string()))?;
    
    use_case.execute(&id).await?;
    
    Ok(Json(()))
}

// Request/Response types

/// Query params para listar usuarios
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

/// Request para crear usuario (admin)
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

/// Request para actualizar usuario
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImpersonateResponse {
    pub access_token: String,
}

/// Response con lista de usuarios
#[derive(Debug, Serialize, ToSchema)]
pub struct ListUsersResponse {
    pub users: Vec<UserResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Response de usuario
#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub email_verified_at: Option<String>,
    pub last_login_at: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
}

impl From<domain::entities::User> for UserResponse {
    fn from(user: domain::entities::User) -> Self {
        let is_active = user.is_active(); // borrow before move
        Self {
            id: user.id.to_string(),
            email: user.email.value().to_string(),
            name: user.name,
            is_active,
            email_verified_at: user.email_verified_at.map(|t| t.to_string()),
            last_login_at: user.last_login_at.map(|t| t.to_string()),
            created_by: user.created_by.map(|id| id.to_string()),
            created_at: user.created_at.to_string(),
        }
    }
}
