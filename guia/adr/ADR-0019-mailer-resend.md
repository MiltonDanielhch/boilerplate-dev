# ADR 0019 — Email: Resend + React Email + LogMailer

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0018 (Apalis — EmailJob), ADR 0002 (Config — RESEND_API_KEY) |

---

## Contexto

El ADR 0018 define `EmailJob` como job asíncrono pero no decide el proveedor de envío.
Necesitamos un servicio de email que cumpla con:

- **Capa gratuita suficiente** — 3.000 emails/mes gratis para el MVP
- **API simple** — sin configurar SPF/DKIM manualmente desde el inicio
- **Deliverability alta** — los emails de bienvenida no deben caer en spam
- **Puerto abstracto** — cambiar el proveedor es cambiar un solo archivo

---

## Decisión

Usar **Resend** como proveedor de email transaccional con plantillas en **React Email**
compiladas a HTML estático, y **LogMailer** en desarrollo.

### Dependencias

```toml
# crates/mailer/Cargo.toml
resend-rs = "0.5"
```

### Puerto de dominio (no cambia con el proveedor)

```rust
// crates/domain/src/ports/mailer.rs
pub trait Mailer: Send + Sync {
    async fn send(&self, email: EmailMessage) -> Result<(), MailerError>;
}

pub struct EmailMessage {
    pub to:       String,
    pub subject:  String,
    pub html:     String,
    pub reply_to: Option<String>,
}
```

### LogMailer — solo en development

```rust
// crates/mailer/src/log_mailer.rs
pub struct LogMailer;

impl Mailer for LogMailer {
    async fn send(&self, email: EmailMessage) -> Result<(), MailerError> {
        // Imprime el email completo en los logs — no envía nada
        tracing::info!(
            to      = %email.to,
            subject = %email.subject,
            html    = %email.html,
            "📧 [LogMailer] Email que se enviaría en producción"
        );
        Ok(())
    }
}
```

### ResendMailer — en producción

```rust
// crates/mailer/src/resend_mailer.rs
use resend_rs::{types::CreateEmailBaseOptions, Resend};

pub struct ResendMailer {
    client: Resend,
    from:   String, // "Boilerplate <noreply@tudominio.com>"
}

impl ResendMailer {
    pub fn new(api_key: &str, from: &str) -> Self {
        Self {
            client: Resend::new(api_key),
            from:   from.to_string(),
        }
    }
}

impl Mailer for ResendMailer {
    async fn send(&self, email: EmailMessage) -> Result<(), MailerError> {
        let options = CreateEmailBaseOptions::new(
            &self.from,
            [email.to.as_str()],
            &email.subject,
        )
        .with_html(&email.html);

        self.client.emails.send(options).await.map_err(MailerError::from)?;
        Ok(())
    }
}
```

### Selección del mailer en el composition root

```rust
// apps/api/src/setup.rs
pub fn build_mailer(config: &AppConfig) -> Arc<dyn Mailer> {
    match config.environment {
        AppEnvironment::Development => Arc::new(LogMailer),
        _ => Arc::new(ResendMailer::new(&config.resend_api_key, &config.mail_from)),
    }
}
```

### Plantillas con React Email (compiladas a HTML)

```tsx
// apps/mailer/emails/welcome.tsx
import { Html, Head, Body, Container, Text, Button } from '@react-email/components';

export default function WelcomeEmail({ email }: { email: string }) {
    return (
        <Html>
            <Head />
            <Body style={{ fontFamily: 'sans-serif' }}>
                <Container>
                    <Text>Bienvenido, {email}</Text>
                    <Button href="https://tudominio.com/login">
                        Ingresar al sistema
                    </Button>
                </Container>
            </Body>
        </Html>
    );
}
```

```bash
# Compilar plantillas a HTML estático — antes del build de Rust
pnpm --filter mailer build
# Genera: apps/mailer/dist/welcome.html, dist/password_reset.html, etc.
```

### Integración con el EmailJob (ADR 0018)

```rust
// apps/api/src/jobs/email_job.rs
pub async fn handle_email_job(job: EmailJob, ctx: JobContext) -> Result<(), JobError> {
    let html = render_template(&job.template)?;

    ctx.data::<Arc<dyn Mailer>>().unwrap()
        .send(EmailMessage {
            to:       job.to,
            subject:  job.subject,
            html,
            reply_to: None,
        })
        .await
        .map_err(JobError::from)
}

fn render_template(template: &EmailTemplate) -> Result<String, JobError> {
    // HTML precompilado por React Email — include_str! lo embebe en el binario
    let html = match template {
        EmailTemplate::Welcome       => include_str!("../../../apps/mailer/dist/welcome.html"),
        EmailTemplate::PasswordReset => include_str!("../../../apps/mailer/dist/password_reset.html"),
        EmailTemplate::LeadWelcome   => include_str!("../../../apps/mailer/dist/lead_welcome.html"),
        EmailTemplate::Notification  => include_str!("../../../apps/mailer/dist/notification.html"),
    };
    Ok(html.to_string())
}
```

---

## Path de migración si Resend no es suficiente

| Escenario | Alternativa | Qué cambia |
|-----------|------------|------------|
| Soberanía total | SMTP propio (Postfix / Stalwart) | Solo el adaptador en `crates/mailer` |
| Volumen alto (>50k/mes) | Postmark o SendGrid | Solo el adaptador |
| Self-hosted | Stalwart Mail Server | Solo el adaptador |

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| SMTP propio desde el inicio | SPF/DKIM/DMARC toma días — innecesario en MVP |
| SendGrid | API más compleja, precio más alto |
| Postmark | Excelente pero sin capa gratuita generosa |
| AWS SES | Barato a escala pero IAM compleja |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la entrega y la experiencia de diseño de correos:

| Herramienta | Propósito en el Mailer |
| :--- | :--- |
| **`lettre`** | **Versatilidad:** La librería de referencia en Rust si se requiere migrar a SMTP o transporte local de correo. |
| **`preview-email`** | **DX de Diseño:** Permite previsualizar las plantillas de React Email en el navegador con hot-reload. |
| **`tracing`** | **Trazabilidad:** Integrar logs de envío con el `request_id` (ADR 0016) para rastrear qué acción disparó cada email. |
| **`sentry`** | **Monitoreo:** Captura de errores de entrega (bounce, spam complaints) reportados por la API de Resend. |

---

## Consecuencias

### ✅ Positivas

- 3.000 emails/mes gratis — suficiente para el MVP
- SPF/DKIM configurados automáticamente por Resend — deliverability alta
- `LogMailer` en development — sin enviar emails reales durante desarrollo
- React Email genera HTML compatible con todos los clientes de correo

### ⚠️ Negativas / Trade-offs

- Dependencia de un servicio externo para emails críticos
  → Mitigado con el puerto `Mailer` abstracto — cambiar proveedor = cambiar un archivo
  → Si Resend cae: los EmailJobs quedan en estado `Failed` en Apalis para reintento
- Las plantillas requieren un paso de compilación antes del build de Rust
  → `pnpm --filter mailer build` corre antes de `cargo build --release` en `just build`
  → Si falta el paso: `cargo build` falla con error claro de `include_str!`
  → En CI: el Containerfile copia `apps/mailer/dist/` antes del build de Rust

### Decisiones derivadas

- `just build` incluye `pnpm --filter mailer build` como primer paso
- `RESEND_API_KEY` y `MAIL_FROM` en `AppConfig` del ADR 0002 como campos requeridos
- En development, el mailer usa `LogMailer` — sin necesidad de credenciales de Resend
- Las plantillas viven en `apps/mailer/emails/` — son código TypeScript, se incluyen en git
