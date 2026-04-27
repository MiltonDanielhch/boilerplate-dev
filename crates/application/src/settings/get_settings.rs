// Ubicación: `crates/application/src/settings/get_settings.rs`

use domain::entities::system_setting::SystemSetting;
use domain::errors::DomainError;
use domain::ports::SettingsRepository;

pub struct GetSettingsUseCase<R: SettingsRepository> {
    settings_repo: R,
}

impl<R: SettingsRepository> GetSettingsUseCase<R> {
    pub fn new(settings_repo: R) -> Self {
        Self { settings_repo }
    }

    pub async fn execute(&self) -> Result<Vec<SystemSetting>, DomainError> {
        self.settings_repo.list_settings().await
    }
}
