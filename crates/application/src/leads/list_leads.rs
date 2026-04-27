// Ubicación: `crates/application/src/leads/list_leads.rs`
//
// Descripción: Caso de uso Listar Leads con filtros.
//
// ADRs relacionados: ADR 0029

use domain::entities::Lead;
use domain::errors::DomainError;
use domain::ports::LeadRepository;
use time::OffsetDateTime;

#[derive(Debug, Clone, Default)]
pub struct ListLeadsInput {
    pub limit: i64,
    pub offset: i64,
    pub search: Option<String>,
    pub status: Option<String>,
    pub from_date: Option<OffsetDateTime>,
    pub to_date: Option<OffsetDateTime>,
}

pub struct ListLeadsUseCase<R: LeadRepository> {
    lead_repo: R,
}

impl<R: LeadRepository> ListLeadsUseCase<R> {
    pub fn new(lead_repo: R) -> Self {
        Self { lead_repo }
    }

    pub async fn execute(&self, input: ListLeadsInput) -> Result<Vec<Lead>, DomainError> {
        self.lead_repo.list(
            input.limit,
            input.offset,
            input.search,
            input.status,
            input.from_date,
            input.to_date,
        ).await
    }
}
