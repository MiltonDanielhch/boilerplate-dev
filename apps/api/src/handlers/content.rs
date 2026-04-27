// Ubicación: `apps/api/src/handlers/content.rs`

use crate::error::ApiResult;
use crate::state::AppState;
use application::content::{ListContentUseCase, UpdateContentUseCase, UpdateContentInput};
use axum::{
    extract::{Path, State},
    response::Json,
};
use domain::value_objects::UserId;
use serde::{Deserialize, Serialize};

/// GET /api/v1/admin/content
pub async fn list(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<ContentBlockResponse>>> {
    let use_case = ListContentUseCase::new(state.content_repo);
    let blocks = use_case.execute().await?;
    
    Ok(Json(blocks.into_iter().map(ContentBlockResponse::from).collect()))
}

/// PUT /api/v1/admin/content/{key}
pub async fn update(
    State(state): State<AppState>,
    claims: crate::middleware::auth::AuthClaims,
    Path(key): Path<String>,
    Json(body): Json<UpdateContentRequest>,
) -> ApiResult<Json<()>> {
    let use_case = UpdateContentUseCase::new(state.content_repo);
    let modified_by = UserId::parse(&claims.user_id)
        .map_err(|_| crate::error::ApiError::Internal("Invalid user_id in token".to_string()))?;

    use_case.execute(UpdateContentInput {
        key,
        content: body.content,
        modified_by,
    }).await?;

    Ok(Json(()))
}

#[derive(Debug, Serialize)]
pub struct ContentBlockResponse {
    pub key: String,
    pub content: String,
    pub content_type: String,
    pub updated_at: String,
}

impl From<domain::entities::content_block::ContentBlock> for ContentBlockResponse {
    fn from(block: domain::entities::content_block::ContentBlock) -> Self {
        Self {
            key: block.key,
            content: block.content,
            content_type: block.content_type,
            updated_at: block.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateContentRequest {
    pub content: String,
}
