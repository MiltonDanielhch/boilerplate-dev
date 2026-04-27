// Ubicación: `crates/application/src/leads/update_lead_status.rs`
//
// Descripción: Caso de uso Actualizar Estado de Lead.

use domain::errors::DomainError;
use domain::ports::LeadRepository;

pub struct UpdateLeadStatusInput {
    pub id: String,
    pub status: String,
}

pub struct UpdateLeadStatusUseCase<R: LeadRepository> {
    lead_repo: R,
}

impl<R: LeadRepository> UpdateLeadStatusUseCase<R> {
    pub fn new(lead_repo: R) -> Self {
        Self { lead_repo }
    }

    pub async fn execute(&self, input: UpdateLeadStatusInput) -> Result<(), DomainError> {
        // Validar status (opcional, mejor con Enum en el futuro)
        let valid_statuses = ["new", "contacted", "qualified", "converted", "archived"];
        if !valid_statuses.contains(&input.status.as_str()) {
            return Err(DomainError::Validation(format!("Invalid status: {}", input.status)));
        }

        self.lead_repo.set_status(&input.id, &input.status).await
    }
}
