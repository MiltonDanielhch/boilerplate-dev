// Ubicación: `crates/application/src/leads/mod.rs`
//
// Descripción: Casos de uso de leads (landing page).
//
// ADRs relacionados: ADR 0001, ADR 0029

pub mod capture_lead;

// Re-exports
pub use capture_lead::CaptureLeadUseCase;
