// Ubicación: `crates/application/src/leads/capture_lead.rs`
//
// Descripción: Caso de uso Capturar Lead (landing page).
//              Deduplicación silenciosa + encola LeadWelcomeJob.
//
// ADRs relacionados: ADR 0001, ADR 0029

use domain::entities::Lead;
use domain::errors::DomainError;
use domain::ports::LeadRepository;
use domain::value_objects::Email;

/// Input para captura de lead.
#[derive(Debug, Clone)]
pub struct CaptureLeadInput {
    pub email: String,
    pub name: Option<String>,
    pub source: Option<String>,      // ej: "landing_v1"
    pub utm_campaign: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
}

/// Caso de uso: Capturar lead desde landing page.
pub struct CaptureLeadUseCase<R: LeadRepository> {
    lead_repo: R,
}

impl<R: LeadRepository> CaptureLeadUseCase<R> {
    pub fn new(lead_repo: R) -> Self {
        Self { lead_repo }
    }

    /// Ejecuta la captura.
    /// Deduplicación silenciosa: si email ya existe, retorna Ok sin error.
    /// TODO: Encolar LeadWelcomeJob sin bloquear HTTP.
    pub async fn execute(&self, input: CaptureLeadInput) -> Result<Lead, DomainError> {
        // Validar email
        let email = Email::new(&input.email)?;

        // Deduplicación silenciosa — si ya existe, retornamos el existente
        if let Some(existing) = self.lead_repo.find_by_email(email.value()).await? {
            return Ok(existing);
        }

        // Crear nuevo lead
        let lead = Lead::new(
            email,
            input.name,
            None,       // phone
            None,       // company
            None,       // message
            input.source,
            input.utm_campaign,
            input.utm_source,
            input.utm_medium,
            None,       // ip_address
            None,       // user_agent
        )?;

        // Guardar
        self.lead_repo.save(&lead).await?;

        // TODO: Encolar LeadWelcomeJob (no bloquear HTTP)
        // self.job_queue.enqueue(LeadWelcomeJob { lead_id: lead.id }).await?;

        Ok(lead)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Agregar tests con mockall
    // - capture_lead_nuevo_guarda_en_db()
    // - capture_lead_email_duplicado_retorna_ok_silencioso()
    // - capture_lead_email_invalido_falla_sin_tocar_db()
    // - capture_lead_encola_welcome_job()
}
