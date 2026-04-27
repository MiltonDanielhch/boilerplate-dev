// Ubicación: `crates/domain/src/entities/system_setting.rs`

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSetting {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
}

impl SystemSetting {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value, description: None }
    }
}
