// Ubicación: `crates/application/src/admin/get_stats.rs`

use domain::errors::DomainError;
use domain::ports::{UserRepository, LeadRepository};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub active_users: i64,
    pub total_leads: i64,
    pub new_leads: i64, // leads con status 'new'
}

pub struct GetAdminStatsUseCase<U: UserRepository, L: LeadRepository> {
    user_repo: U,
    lead_repo: L,
}

impl<U: UserRepository, L: LeadRepository> GetAdminStatsUseCase<U, L> {
    pub fn new(user_repo: U, lead_repo: L) -> Self {
        Self { user_repo, lead_repo }
    }

    pub async fn execute(&self) -> Result<AdminStats, DomainError> {
        // En una implementación real, esto debería usar queries optimizadas de conteo
        // Por ahora usamos lo que tenemos o mockeamos la lógica de conteo
        
        let users = self.user_repo.list(1000, 0, None, None, None).await?;
        let total_users = users.len() as i64;
        let active_users = users.iter().filter(|u| u.is_active()).count() as i64;

        // Para leads, usamos el list con filtro
        let leads = self.lead_repo.list(1000, 0, None, None, None, None).await?;
        let total_leads = leads.len() as i64;
        let new_leads = leads.iter().filter(|l| l.status == "new").count() as i64;

        Ok(AdminStats {
            total_users,
            active_users,
            total_leads,
            new_leads,
        })
    }
}
