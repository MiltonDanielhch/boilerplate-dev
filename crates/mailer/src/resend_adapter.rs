// Ubicación: `crates/mailer/src/resend_adapter.rs`
//
// Descripción: Adaptadores de email: LogMailer (dev) y Resend (prod).
//
// ADRs relacionados: ADR 0019

use crate::ports::MailerPort;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LogMailer;

impl LogMailer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LogMailer {
    fn default() -> Self {
        Self::new()
    }
}

impl MailerPort for LogMailer {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        tracing::info!(
            target: "mailer::log",
            to = %to,
            subject = %subject,
            "📧 [LogMailer] Email que se enviaría en producción:\n{}",
            body
        );
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ResendMailer {
    api_key: String,
    from_email: String,
    from_name: String,
}

impl ResendMailer {
    pub fn new(api_key: String, from_email: String, from_name: String) -> Self {
        Self {
            api_key,
            from_email,
            from_name,
        }
    }
}

impl MailerPort for ResendMailer {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let client = reqwest::blocking::Client::new();

        let response = client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "from": format!("{} <{}>", self.from_name, self.from_email),
                "to": to,
                "subject": subject,
                "html": body
            }))
            .send()
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            tracing::info!(to = %to, subject = %subject, "Email enviado exitosamente via Resend");
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            tracing::error!(to = %to, status = %status, error = %error_text, "Falló envio de email");
            Err(format!("Resend API error: {} - {}", status, error_text))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub to: String,
    pub subject: String,
    pub html_body: String,
}

pub fn build_mailer(config: &MailerConfig) -> Box<dyn MailerPort> {
    match config.provider.as_str() {
        "resend" => Box::new(ResendMailer::new(
            config.api_key.clone(),
            config.from_email.clone(),
            config.from_name.clone(),
        )),
        _ => Box::new(LogMailer::new()),
    }
}

#[derive(Debug, Clone)]
pub struct MailerConfig {
    pub provider: String,
    pub api_key: String,
    pub from_email: String,
    pub from_name: String,
}

impl MailerConfig {
    pub fn from_env() -> Self {
        Self {
            provider: std::env::var("MAILER_PROVIDER").unwrap_or_else(|_| "log".to_string()),
            api_key: std::env::var("RESEND_API_KEY").unwrap_or_default(),
            from_email: std::env::var("MAILER_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@example.com".to_string()),
            from_name: std::env::var("MAILER_FROM_NAME")
                .unwrap_or_else(|_| "Boilerplate".to_string()),
        }
    }
}
