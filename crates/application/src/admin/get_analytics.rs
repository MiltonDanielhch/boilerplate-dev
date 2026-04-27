// Ubicación: `crates/application/src/admin/get_analytics.rs`

use domain::errors::DomainError;
use domain::ports::{UserRepository, LeadRepository};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DataPoint {
    pub label: String,
    pub value: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminAnalytics {
    pub users_over_time: Vec<DataPoint>,
    pub leads_over_time: Vec<DataPoint>,
    pub leads_by_status: Vec<DataPoint>,
}

pub struct GetAdminAnalyticsUseCase<U: UserRepository, L: LeadRepository> {
    user_repo: U,
    lead_repo: L,
}

impl<U: UserRepository, L: LeadRepository> GetAdminAnalyticsUseCase<U, L> {
    pub fn new(user_repo: U, lead_repo: L) -> Self {
        Self { user_repo, lead_repo }
    }

    pub async fn execute(&self, days: i64) -> Result<AdminAnalytics, DomainError> {
        let user_counts = self.user_repo.get_counts_by_date(days).await?;
        let lead_counts = self.lead_repo.get_counts_by_date(days).await?;
        let lead_status = self.lead_repo.get_counts_by_status().await?;

        Ok(AdminAnalytics {
            users_over_time: user_counts.into_iter().map(|(label, value)| DataPoint { label, value }).collect(),
            leads_over_time: lead_counts.into_iter().map(|(label, value)| DataPoint { label, value }).collect(),
            leads_by_status: lead_status.into_iter().map(|(label, value)| DataPoint { label, value }).collect(),
        })
    }
}
