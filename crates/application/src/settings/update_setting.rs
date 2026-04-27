// Ubicación: `crates/application/src/settings/update_setting.rs`

use domain::entities::system_setting::SystemSetting;
use domain::errors::DomainError;
use domain::ports::SettingsRepository;

pub struct UpdateSettingInput {
    pub key: String,
    pub value: String,
}

pub struct UpdateSettingUseCase<R: SettingsRepository> {
    settings_repo: R,
}

impl<R: SettingsRepository> UpdateSettingUseCase<R> {
    pub fn new(settings_repo: R) -> Self {
        Self { settings_repo }
    }

    pub async fn execute(&self, input: UpdateSettingInput) -> Result<(), DomainError> {
        let mut setting = self.settings_repo.get_setting(&input.key).await?
            .ok_or_else(|| DomainError::not_found("System setting"))?;

        setting.value = input.value;

        self.settings_repo.save_setting(&setting).await
    }
}
