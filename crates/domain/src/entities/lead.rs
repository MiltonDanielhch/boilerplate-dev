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
    pub phone: Option<String>,
    pub company: Option<String>,
    pub message: Option<String>,
    pub source: Option<String>, // ej: "landing_v1", "blog_post_x"
    pub utm_campaign: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: String, // new, contacted, qualified, converted, archived
    pub is_contacted: bool,
    pub contact_notes: Option<String>,
    pub contacted_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

impl Lead {
    /// Crea un nuevo lead desde la landing page.
    pub fn new(
        email: Email,
        name: Option<String>,
        phone: Option<String>,
        company: Option<String>,
        message: Option<String>,
        source: Option<String>,
        utm_campaign: Option<String>,
        utm_source: Option<String>,
        utm_medium: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<Self, crate::errors::DomainError> {
        Ok(Self {
            id: Uuid::now_v7().to_string(),
            email,
            name,
            phone,
            company,
            message,
            source,
            utm_campaign,
            utm_source,
            utm_medium,
            ip_address,
            user_agent,
            status: "new".to_string(),
            is_contacted: false,
            contact_notes: None,
            contacted_at: None,
            created_at: OffsetDateTime::now_utc(),
        })
    }

    /// Marca como contactado.
    pub fn mark_contacted(&mut self, notes: Option<String>) {
        self.is_contacted = true;
        self.contact_notes = notes;
        self.contacted_at = Some(OffsetDateTime::now_utc());
    }
}
