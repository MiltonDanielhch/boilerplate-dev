// Ubicación: `crates/mailer/src/resend_adapter.rs`
//
// Descripción: Adaptador concreto de Resend.
//
// ADRs relacionados: ADR 0019

use crate::ports::MailerPort;

pub struct ResendAdapter;

impl MailerPort for ResendAdapter {
    fn send(&self, to: &str, subject: &str, _body: &str) -> Result<(), String> {
        println!("Sending email to {}: {}", to, subject);
        Ok(())
    }
}
