// Ubicación: `crates/events/src/lib.rs`
//
// Descripción: Eventos con NATS JetStream. 🟡 Fase 2 — NO usar hasta que
//              exista el problema real de desacoplamiento.
//
// ADRs relacionados: ADR 0001, ADR 0025 (NATS)
// Estado: 🟡 Diferido — solo se activa en Fase 2

pub mod publisher;
pub mod subscriber;
