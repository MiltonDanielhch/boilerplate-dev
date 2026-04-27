// Ubicación: `crates/domain/src/ports/settings_repository.rs`

use crate::entities::system_setting::SystemSetting;
use crate::errors::DomainError;
use std::future::Future;

pub trait SettingsRepository: Send + Sync {
    fn get_setting(&self, key: &str) -> impl Future<Output = Result<Option<SystemSetting>, DomainError>> + Send;
    fn save_setting(&self, setting: &SystemSetting) -> impl Future<Output = Result<(), DomainError>> + Send;
    fn list_settings(&self) -> impl Future<Output = Result<Vec<SystemSetting>, DomainError>> + Send;
}
