// Ubicación: `crates/domain/src/ports/mailer.rs`
//
// Descripción: Puerto (trait) para envío de emails.
//              Implementado por adaptadores (Resend, SMTP, etc.).
//
// ADRs relacionados: ADR 0001, ADR 0019

use crate::errors::DomainError;
use async_trait::async_trait;

/// Email con contenido HTML y/o texto plano.
#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
}

impl EmailMessage {
    pub fn new(to: String, subject: String) -> Self {
        Self {
            to,
            subject,
            html_body: None,
            text_body: None,
        }
    }

    pub fn with_html(mut self, html: String) -> Self {
        self.html_body = Some(html);
        self
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text_body = Some(text);
        self
    }
}

/// Puerto para envío de emails.
#[async_trait]
pub trait Mailer: Send + Sync {
    /// Envía un email.
    async fn send(&self, message: &EmailMessage) -> Result<(), DomainError>;

    /// Envía email de verificación de cuenta.
    async fn send_verification_email(
        &self,
        to: &str,
        name: &str,
        verification_url: &str,
    ) -> Result<(), DomainError>;

    /// Envía email de reset de contraseña.
    async fn send_password_reset(
        &self,
        to: &str,
        reset_url: &str,
    ) -> Result<(), DomainError>;
}
