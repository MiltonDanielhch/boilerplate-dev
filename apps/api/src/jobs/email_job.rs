// Ubicación: `apps/api/src/jobs/email_job.rs`
//
// Descripción: Job para envío de emails asíncronos.
//
// ADRs relacionados: ADR 0018, ADR 0019

use apalis::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailJob {
    pub to: String,
    pub subject: String,
    pub template: EmailTemplate,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailTemplate {
    Welcome,
    PasswordReset,
    LeadWelcome,
    Notification,
}

impl std::fmt::Display for EmailTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailTemplate::Welcome => write!(f, "welcome"),
            EmailTemplate::PasswordReset => write!(f, "password_reset"),
            EmailTemplate::LeadWelcome => write!(f, "lead_welcome"),
            EmailTemplate::Notification => write!(f, "notification"),
        }
    }
}

#[derive(Clone)]
pub struct EmailJobHandler {
    mailer: Box<dyn crate::mailer::ports::MailerPort>,
}

impl EmailJobHandler {
    pub fn new(mailer: Box<dyn crate::mailer::ports::MailerPort>) -> Self {
        Self { mailer }
    }
}

#[async_trait::async_trait]
impl Job for EmailJob {
    async fn run(&self, _ctx: JobContext) -> Result<(), apalis::prelude::JobError> {
        let html_body = self.render_template();
        
        tracing::info!(
            to = %self.to,
            template = %self.template,
            "Processing email job"
        );

        self.mailer
            .send(&self.to, &self.subject, &html_body)
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to send email");
                JobError::Unexpected(e.to_string())
            })?;

        tracing::info!(
            to = %self.to,
            template = %self.template,
            "Email job completed successfully"
        );

        Ok(())
    }
}

impl EmailJob {
    fn render_template(&self) -> String {
        let context_json = serde_json::to_string(&self.context).unwrap_or_default();
        
        match self.template {
            EmailTemplate::Welcome => format!(
                r#"<html><body><h1>Welcome!</h1><p>Thanks for registering. Context: {}</p></body></html>"#,
                context_json
            ),
            EmailTemplate::PasswordReset => format!(
                r#"<html><body><h1>Reset Password</h1><p>Click here to reset. Context: {}</p></body></html>"#,
                context_json
            ),
            EmailTemplate::LeadWelcome => format!(
                r#"<html><body><h1>Welcome!</h1><p>Thanks for your interest. Context: {}</p></body></html>"#,
                context_json
            ),
            EmailTemplate::Notification => format!(
                r#"<html><body><h1>Notification</h1><p>You have a new notification. Context: {}</p></body></html>"#,
                context_json
            ),
        }
    }
}