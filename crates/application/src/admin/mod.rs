// Ubicación: `crates/application/src/admin/mod.rs`

pub mod get_analytics;
pub mod get_stats;

pub use get_analytics::{GetAdminAnalyticsUseCase, AdminAnalytics, DataPoint};
pub use get_stats::{GetAdminStatsUseCase, AdminStats};
