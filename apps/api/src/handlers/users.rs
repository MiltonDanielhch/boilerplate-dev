// Ubicación: `apps/api/src/handlers/users.rs`
//
// Descripción: Handlers CRUD para usuarios.
//
// ADRs relacionados: ADR 0003 (Axum), ADR 0006 (Soft Delete)

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use application::users::{GetUserUseCase, ListUsersUseCase, SoftDeleteUserUseCase, UpdateUserUseCase};
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use domain::value_objects::UserId;
use serde::{Deserialize, Serialize};

/// GET /api/v1/users/:id
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
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> ApiResult<Json<ListUsersResponse>> {
    let use_case = ListUsersUseCase::new(state.user_repo);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let input = application::users::ListUsersInput { limit, offset };
    
    let users = use_case.execute(input).await?;
    
    Ok(Json(ListUsersResponse {
        users: users.into_iter().map(UserResponse::from).collect(),
        limit,
        offset,
    }))
}

/// POST /api/v1/users
pub async fn create(
    State(_state): State<AppState>,
    Json(_body): Json<CreateUserRequest>,
) -> ApiResult<Json<UserResponse>> {
    // TODO: Implementar cuando tengamos el caso de uso completo
    Err(ApiError::Internal("Not implemented".to_string()))
}

/// PUT /api/v1/users/:id
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateUserRequest>,
) -> ApiResult<Json<UserResponse>> {
    let use_case = UpdateUserUseCase::new(state.user_repo);
    let id = UserId::parse(&id).map_err(|_| ApiError::Validation("Invalid user ID".to_string()))?;
    let input = application::users::UpdateUserInput { name: body.name };
    
    let user = use_case.execute(&id, input).await?;
    
    Ok(Json(UserResponse::from(user)))
}

/// DELETE /api/v1/users/:id — Soft delete (ADR 0006)
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

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListUsersResponse {
    pub users: Vec<UserResponse>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

impl From<domain::entities::User> for UserResponse {
    fn from(user: domain::entities::User) -> Self {
        let is_active = user.is_active(); // borrow before move
        Self {
            id: user.id.to_string(),
            email: user.email.value().to_string(),
            name: user.name, // move happens here
            is_active,
            created_at: user.created_at.to_string(),
        }
    }
}
