// Ubicación: `crates/application/src/settings/mod.rs`

pub mod get_settings;
pub mod update_setting;

pub use get_settings::GetSettingsUseCase;
pub use update_setting::{UpdateSettingInput, UpdateSettingUseCase};
