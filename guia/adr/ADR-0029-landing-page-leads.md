# ADR 0029 — Landing Page: Captura de Leads + SEO

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0022 (Astro SSR), ADR 0018 (Apalis — LeadWelcomeJob), ADR 0009 (Rate Limiting) |

---

## Contexto

El boilerplate necesita una landing page pública que cumpla con:

- **Captura de leads** — formulario de registro de interés antes del lanzamiento
- **SEO nativo** — HTML desde el servidor, sin JavaScript requerido para indexar
- **Core Web Vitals excelentes** — LCP <2.5s, CLS <0.1
- **Anti-spam** — protección contra bots y envíos masivos
- **Sin duplicar el sistema de usuarios** — los leads no son usuarios autenticados

---

## Decisión

Usar **Astro SSR** para la landing con un formulario de captura de leads respaldado
por un endpoint `/api/v1/leads` con rate limiting estricto y `LeadWelcomeJob` en Apalis.

### Entidad `Lead` en el dominio

```rust
// crates/domain/src/entities/lead.rs
#[derive(Debug, Clone)]
pub struct Lead {
    pub id:         LeadId,
    pub email:      Email,
    pub name:       Option<String>,
    pub source:     LeadSource,   // "landing", "referral", "social"
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub enum LeadSource {
    Landing,
    Referral,
    Social,
}
```

### Caso de uso `CaptureLead`

```rust
// crates/application/src/use_cases/leads/capture_lead.rs
pub async fn execute(&self, input: CaptureLeadInput) -> Result<(), AppError> {
    let email = Email::new(&input.email)?;

    // Deduplicación silenciosa — si ya existe, retornar OK sin error
    // No confirmar al atacante si el email está en la base
    if self.leads.find_by_email(&email).await?.is_some() {
        return Ok(());
    }

    let lead = Lead::new(email, input.name, LeadSource::Landing);
    self.leads.save(&lead).await?;

    // Encolar email de bienvenida — no bloquea la respuesta HTTP
    if let Err(e) = self.jobs.push(EmailJob {
        to:       lead.email.to_string(),
        subject:  "Gracias por tu interés".into(),
        template: EmailTemplate::LeadWelcome,
        context:  serde_json::json!({ "email": lead.email }),
    }).await {
        tracing::warn!(error = ?e, "failed to enqueue lead welcome email");
    }

    Ok(())
}
```

### Endpoint con rate limiting estricto — ADR 0009

```rust
// crates/infrastructure/src/http/handlers/leads.rs
#[utoipa::path(
    post,
    path = "/api/v1/leads",
    request_body = CaptureLeadRequest,
    responses(
        (status = 200, description = "Lead registrado"),
        (status = 429, description = "Rate limit excedido"),
    ),
    tag = "leads",
)]
pub async fn capture_lead(
    State(state): State<AppState>,
    Json(body):   Json<CaptureLeadRequest>,
) -> Result<StatusCode, AppError> {
    CaptureLead::new(&state).execute(body.into()).await?;
    Ok(StatusCode::OK)
}
```

```rust
// Rate limit de 3 req/min para /api/v1/leads — en el router
let leads_routes = Router::new()
    .route("/api/v1/leads", post(capture_lead))
    .layer(leads_rate_limit()); // tower-governor — ADR 0009
```

### Formulario en Svelte 5 — anti-spam con honeypot

```svelte
<!-- apps/web/src/components/landing/LeadForm.svelte -->
<script lang="ts">
    import { type } from 'arktype';

    const LeadSchema = type({
        email:   'string.email',
        name:    'string?',
        honeypot: 'string?',  // Si está relleno → es un bot
    });

    let email    = $state('');
    let name     = $state('');
    let honeypot = $state('');  // Campo oculto con CSS — los bots lo rellenan
    let status   = $state<'idle' | 'loading' | 'success' | 'error'>('idle');

    async function handleSubmit() {
        // Anti-spam: si el honeypot tiene valor, silenciosamente no enviar
        if (honeypot) return;

        const result = LeadSchema({ email, name, honeypot });
        if (result instanceof type.errors) return;

        status = 'loading';
        try {
            await fetch('/api/v1/leads', {
                method:  'POST',
                headers: { 'Content-Type': 'application/json' },
                body:    JSON.stringify({ email, name }),
            });
            status = 'success';
        } catch {
            status = 'error';
        }
    }
</script>

{#if status === 'success'}
    <div class="success">¡Gracias! Te avisaremos cuando lancemos.</div>
{:else}
    <form onsubmit|preventDefault={handleSubmit}>
        <!-- Honeypot: oculto con CSS — los bots lo ven y lo rellenan -->
        <input
            type="text"
            bind:value={honeypot}
            style="display:none"
            autocomplete="off"
            tabindex="-1"
        />
        <input type="email" bind:value={email} placeholder="tu@email.com" required />
        <input type="text"  bind:value={name}  placeholder="Tu nombre (opcional)" />
        <button type="submit" disabled={status === 'loading'}>
            {status === 'loading' ? 'Enviando...' : 'Notificarme del lanzamiento'}
        </button>
    </form>
{/if}
```

### Estructura de la landing page

```astro
---
// apps/web/src/pages/index.astro
// HTML puro desde el servidor — sin JavaScript requerido para ver el contenido
---
<LandingLayout
    title="boilerplate — Rust · Axum · Svelte 5"
    description="Boilerplate de alto rendimiento para construir SaaS desde el día uno"
>
    <!-- Secciones de la landing -->
    <Hero />
    <Problema />
    <Solucion />
    <Features />
    <SocialProof />
    <CallToAction>
        <LeadForm client:load />
    </CallToAction>
    <Footer />
</LandingLayout>
```

### Layout de la landing — separado del dashboard

```astro
---
// apps/web/src/layouts/LandingLayout.astro
// Sin sidebar, sin verificación de sesión
// SEO completo: meta tags, Open Graph, sitemap
---
<!DOCTYPE html>
<html lang="es">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{Astro.props.title}</title>
    <meta name="description" content={Astro.props.description} />
    <meta property="og:title"       content={Astro.props.title} />
    <meta property="og:description" content={Astro.props.description} />
    <!-- Sitemap generado automáticamente por @astrojs/sitemap -->
  </head>
  <body>
    <slot />
  </body>
</html>
```

---

## Tabla en la base de datos

```sql
-- Incluida en las migraciones como migración 7 (después de las 6 base)
-- o como migración separada según el proyecto

CREATE TABLE IF NOT EXISTS leads (
    id         TEXT     PRIMARY KEY NOT NULL,
    email      TEXT     NOT NULL UNIQUE,
    name       TEXT,
    source     TEXT     NOT NULL DEFAULT 'landing',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_leads_email ON leads(email);
CREATE INDEX IF NOT EXISTS idx_leads_source ON leads(source);
```

---

## Core Web Vitals — objetivos

| Métrica | Objetivo | Cómo se logra |
|---------|----------|---------------|
| LCP | <2.5s | HTML desde Astro SSR — sin JS bloqueante |
| CLS | <0.1 | Tailwind con dimensiones explícitas en imágenes |
| FID / INP | <100ms | Svelte 5 compila sin runtime — <5KB por componente |
| TTFB | <200ms | Axum + SQLite en VPS — sin cold starts |

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| Landing en servicio externo (Framer, Webflow) | Dos sistemas a mantener — Astro SSR puede hacer lo mismo |
| Formulario con Google Forms | Sin control de datos, sin email propio, datos en Google |
| Guardar leads como usuarios | Los leads no tienen password ni sesión — entidad diferente |
| CAPTCHA (reCAPTCHA) | UX degradada — honeypot + rate limiting es suficiente para MVP |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la conversión y el rendimiento de la Landing Page:

| Herramienta | Propósito en la Landing |
| :--- | :--- |
| **`Cloudflare Turnstile`** | **Anti-spam invisible:** Alternativa a CAPTCHA que no degrada la UX y detiene bots avanzados. |
| **`Partytown`** | **Rendimiento:** Ejecuta scripts de terceros en Web Workers para no bloquear el hilo principal. |
| **`Astro Image`** | **Optimización Visual:** Generación automática de imágenes en formato AVIF/WebP para carga instantánea. |
| **`PostHog`** | **Analytics:** Análisis de funnels y comportamiento de usuarios para optimizar la conversión de leads. |

---

## Consecuencias

### ✅ Positivas

- Los leads se capturan en la misma DB que el resto del sistema — sin servicios externos
- `LeadWelcomeJob` envía email automáticamente via Resend — ADR 0019
- La landing es SSR puro — indexada por Google sin JavaScript
- `Lead` es una entidad del dominio separada de `User` — sin contaminar el sistema de auth

### ⚠️ Negativas / Trade-offs

- Los leads no pueden iniciar sesión — si se convierten en usuarios, hay que migrarlos
  → Proceso: `INSERT INTO users SELECT id, email FROM leads WHERE email = $1`
  → Luego enviar email de creación de password — un solo flujo conocido
- El honeypot no detiene todos los bots — solo los más simples
  → Suficiente para MVP — el rate limiting de 3 req/min es la defensa principal
  → En Fase 2: evaluar Cloudflare Turnstile si el spam es un problema real

### Decisiones derivadas

- El endpoint `/api/v1/leads` tiene rate limiting de 3 req/min por IP — ADR 0009
- La deduplicación es silenciosa — retorna 200 aunque el email ya exista
- `CleanupJob` puede limpiar leads no convertidos después de X meses — ADR 0018
- `LandingLayout.astro` y `DashboardLayout.astro` son completamente independientes
