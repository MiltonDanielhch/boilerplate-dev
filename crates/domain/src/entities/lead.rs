// Ubicación: `crates/domain/src/entities/lead.rs`
//
// Descripción: Entidad Lead para landing page (ADR 0029).
//              Captura de emails de interesados.
//
// ADRs relacionados: ADR 0001, ADR 0029

use crate::value_objects::Email;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Entidad Lead del dominio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lead {
    pub id: String,     // UUID v7
    pub email: Email,
    pub name: Option<String>,
    pub source: Option<String>, // ej: "landing_v1", "blog_post_x"
    pub utm_campaign: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub is_contacted: bool,
    pub notes: Option<String>,
    pub created_at: OffsetDateTime,
}

impl Lead {
    /// Crea un nuevo lead desde la landing page.
    pub fn new(
        email: Email,
        name: Option<String>,
        source: Option<String>,
        utm_campaign: Option<String>,
        utm_source: Option<String>,
        utm_medium: Option<String>,
    ) -> Result<Self, crate::errors::DomainError> {
        Ok(Self {
            id: Uuid::now_v7().to_string(),
            email,
            name,
            source,
            utm_campaign,
            utm_source,
            utm_medium,
            is_contacted: false,
            notes: None,
            created_at: OffsetDateTime::now_utc(),
        })
    }

    /// Marca como contactado.
    pub fn mark_contacted(&mut self, notes: Option<String>) {
        self.is_contacted = true;
        if let Some(n) = notes {
            self.notes = Some(n);
        }
    }
}
