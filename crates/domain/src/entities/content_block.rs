// Ubicación: `crates/domain/src/entities/content_block.rs`
//
// Descripción: Entidad para bloques de contenido dinámico (CMS básico).

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::value_objects::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    pub key: String, // ej: "hero_title", "pricing_description"
    pub content: String,
    pub content_type: String, // text, markdown, html
    pub last_modified_by: Option<UserId>,
    pub updated_at: OffsetDateTime,
}

impl ContentBlock {
    pub fn new(key: String, content: String, content_type: String) -> Self {
        Self {
            key,
            content,
            content_type,
            last_modified_by: None,
            updated_at: OffsetDateTime::now_utc(),
        }
    }
}
