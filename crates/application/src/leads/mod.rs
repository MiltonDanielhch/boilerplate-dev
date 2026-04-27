// Ubicación: `crates/application/src/leads/mod.rs`
//
// Descripción: Casos de uso de leads (landing page).
//
// ADRs relacionados: ADR 0001, ADR 0029

pub mod capture_lead;
pub mod list_leads;
pub mod update_lead_status;

pub use capture_lead::{CaptureLeadInput, CaptureLeadUseCase};
pub use list_leads::{ListLeadsInput, ListLeadsUseCase};
pub use update_lead_status::{UpdateLeadStatusInput, UpdateLeadStatusUseCase};
