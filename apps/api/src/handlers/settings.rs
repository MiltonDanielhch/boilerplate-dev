// Ubicación: `apps/api/src/handlers/settings.rs`

use crate::error::ApiResult;
use crate::state::AppState;
use application::settings::{GetSettingsUseCase, UpdateSettingUseCase, UpdateSettingInput};
use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};

/// GET /api/v1/admin/settings
pub async fn list(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<SettingResponse>>> {
    let use_case = GetSettingsUseCase::new(state.settings_repo);
    let settings = use_case.execute().await?;
    
    Ok(Json(settings.into_iter().map(SettingResponse::from).collect()))
}

/// PUT /api/v1/admin/settings/{key}
pub async fn update(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(body): Json<UpdateSettingRequest>,
) -> ApiResult<Json<()>> {
    let use_case = UpdateSettingUseCase::new(state.settings_repo);
    
    use_case.execute(UpdateSettingInput {
        key,
        value: body.value,
    }).await?;

    Ok(Json(()))
}

#[derive(Debug, Serialize)]
pub struct SettingResponse {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
}

impl From<domain::entities::system_setting::SystemSetting> for SettingResponse {
    fn from(s: domain::entities::system_setting::SystemSetting) -> Self {
        Self {
            key: s.key,
            value: s.value,
            description: s.description,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingRequest {
    pub value: String,
}
