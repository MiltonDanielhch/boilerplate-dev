// Ubicación: `crates/mailer/src/ports.rs`
//
// Descripción: Puerto (trait) del mailer — define el contrato.
//
// ADRs relacionados: ADR 0001

pub trait MailerPort {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), String>;
}
