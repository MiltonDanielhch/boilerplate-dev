// Ubicación: `crates/application/src/audit/list_audit.rs`

use domain::entities::AuditLog;
use domain::errors::DomainError;
use domain::ports::AuditRepository;
use time::OffsetDateTime;

#[derive(Debug, Clone, Default)]
pub struct ListAuditInput {
    pub limit: i64,
    pub offset: i64,
    pub user_id: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub from_date: Option<OffsetDateTime>,
    pub to_date: Option<OffsetDateTime>,
}

pub struct ListAuditUseCase<R: AuditRepository> {
    audit_repo: R,
}

impl<R: AuditRepository> ListAuditUseCase<R> {
    pub fn new(audit_repo: R) -> Self {
        Self { audit_repo }
    }

    pub async fn execute(&self, input: ListAuditInput) -> Result<Vec<AuditLog>, DomainError> {
        self.audit_repo.list(
            input.limit,
            input.offset,
            input.user_id,
            input.resource,
            input.action,
            input.from_date,
            input.to_date,
        ).await
    }
}
