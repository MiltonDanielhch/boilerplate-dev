# Roadmap — Landing Page (Back + Front juntos)

> **Stack:** Astro 5 SSR · Svelte 5 · Tailwind v4 · Paraglide JS · Resend · Apalis
>
> **ADRs:** 0022 (Frontend) · 0029 (Landing + Leads) · 0009 (Rate Limit) · 0019 (Email)
>
> **Pre-requisitos:**
> - Frontend FE.I completado (`ROADMAP-FRONTEND.md`)
> - Backend Bloque II (endpoint /api/v1/leads)
> - Backend Bloque V.3 (Resend mailer)

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| L.1 | Dominio + DB (entidad Lead) | 0% |
| L.2 | Endpoint backend | 0% |
| L.3 | Layout + estructura | 0% |
| L.4 | Contenido y secciones | 0% |
| L.5 | Formulario de leads | 0% |
| L.6 | SEO + performance | 0% |
| L.7 | Tests + deploy | 0% |

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

## L.1 — Dominio + DB (entidad Lead) — ADR 0029

> **Referencia:** ADR 0029 (Landing + Leads), ADR 0006 (Soft Delete), ADR 0018 (Jobs), ADR 0019 (Email), docs/02-STACK.md L274-295, docs/01-ARCHITECTURE.md L139-164

### Backend — Dominio

```
[ ] crates/domain/src/entities/lead.rs:
    └─ Ref: docs/03-STRUCTURE.md L191-194
    [ ] struct Lead { id: LeadId, email: Email, name: Option<String>, source: LeadSource, created_at }
    [ ] enum LeadSource { Landing, Referral, Social }
        └─ Ref: ADR 0029
    [ ] impl Lead: new(email, name, source)

[ ] crates/domain/src/ports/lead_repository.rs:
    └─ Ref: docs/03-STRUCTURE.md L195-198
    [ ] trait LeadRepository: Send + Sync
    [ ] save(lead: &Lead) → Result<(), DomainError>
        └─ Ref: ADR 0007
    [ ] find_by_email(email: &Email) → Result<Option<Lead>, DomainError>
        └─ Ref: ADR 0029
```

### Backend — Repositorio

> **Referencia:** ADR 0029, docs/03-STRUCTURE.md L304-307, docs/02-STACK.md L155-170

```
[ ] crates/database/src/models/lead_row.rs:
    └─ Ref: docs/03-STRUCTURE.md L304
    [ ] struct LeadRow con mapeo exacto de columnas

[ ] crates/database/src/repositories/sqlite_lead_repository.rs:
    └─ Ref: docs/03-STRUCTURE.md L305-307
    [ ] impl LeadRepository
    [ ] save() con INSERT OR IGNORE (idempotente)
        └─ Ref: ADR 0029 — no revelar duplicados
    [ ] find_by_email() con índice idx_leads_email
```

### Backend — Migración

> **Referencia:** ADR 0004 (SQLite), docs/03-STRUCTURE.md L305, docs/02-STACK.md L155-170

```
[ ] data/migrations/{timestamp}_create_leads.sql:
    └─ Ref: docs/03-STRUCTURE.md L305
    [ ] CREATE TABLE IF NOT EXISTS leads (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL UNIQUE,
            name TEXT,
            source TEXT NOT NULL DEFAULT 'landing',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        └─ Ref: ADR 0029
    [ ] CREATE INDEX IF NOT EXISTS idx_leads_email ON leads(email)
    [ ] CREATE INDEX IF NOT EXISTS idx_leads_source ON leads(source)
        └─ Ref: ADR 0004 — índices para queries frecuentes

[ ] just migrate → "Applied {timestamp}/migrate create_leads"
    └─ Ref: ADR 0012, docs/02-STACK.md L413-415
```

### Backend — Caso de uso

> **Referencia:** ADR 0029, ADR 0018 (Jobs), ADR 0019 (Email), docs/01-ARCHITECTURE.md L223-229

```
[ ] crates/application/src/use_cases/leads/capture_lead.rs:
    └─ Ref: docs/03-STRUCTURE.md L237
    [ ] CaptureLeadInput { email: String, name: Option<String> }
    [ ] Email::new() valida + normaliza
        └─ Ref: docs/02-STACK.md L88 — Email como value object
    [ ] find_by_email() → retorna Ok(()) silenciosamente si ya existe
        (No revelar si el email está registrado)
        └─ Ref: ADR 0029 — privacidad de leads
    [ ] Lead::new(email, name, LeadSource::Landing)
    [ ] lead_repo.save()
    [ ] Encolar EmailJob:LeadWelcome (no bloquea HTTP)
        └─ Ref: ADR 0018, ADR 0019, docs/02-STACK.md L274-295
    [ ] retorna Ok(())

[ ] Test:
    └─ Ref: ADR 0010, docs/02-STACK.md L429-443
    [ ] lead_nuevo_se_guarda()
    [ ] lead_duplicado_retorna_ok_silencioso()   ← sin error
        └─ Ref: ADR 0029
    [ ] email_invalido_retorna_error()
        └─ Ref: ADR 0007
```

---

## L.2 — Endpoint backend — ADR 0029, 0009, 0021

> **Referencia:** ADR 0029 (Landing + Leads), ADR 0009 (Rate Limit), ADR 0021 (OpenAPI), docs/02-STACK.md L239-250, docs/03-STRUCTURE.md L273-276

```
[ ] crates/infrastructure/src/http/handlers/leads.rs:
    └─ Ref: docs/03-STRUCTURE.md L273-276
    [ ] CaptureLeadRequest { email: String, name: Option<String> }
    [ ] #[derive(Deserialize, ToSchema)]
        └─ Ref: ADR 0021, docs/02-STACK.md L246

    [ ] #[utoipa::path(
            post,
            path = "/api/v1/leads",
            request_body = CaptureLeadRequest,
            responses(
                (status = 200, description = "Lead registrado"),
                (status = 400, description = "Email inválido"),
                (status = 429, description = "Rate limit excedido"),
            ),
            tag = "leads",
        )]
        └─ Ref: ADR 0021, docs/02-STACK.md L246
    [ ] pub async fn capture_lead(...) → llama CaptureLeadUseCase → 200

[ ] En router.rs:
    └─ Ref: docs/03-STRUCTURE.md L278
    [ ] POST /api/v1/leads con rate limit 3 req/min (ADR 0009)
        └─ Ref: docs/02-STACK.md L143, ADR 0009
    [ ] Sin auth_middleware (la landing es pública)
        └─ Ref: ADR 0008 — solo endpoints /api/v1/* protegidos

[ ] Email de bienvenida en apps/mailer/emails/lead_welcome.tsx:
    └─ Ref: docs/03-STRUCTURE.md L503, ADR 0019
    [ ] Título: "¡Gracias por tu interés!"
    [ ] CTA para compartir o esperar novedades

[ ] Test de endpoint:
    └─ Ref: ADR 0010, docs/02-STACK.md L429-443
    [ ] lead_valido_retorna_200()
    [ ] email_invalido_retorna_400()
        └─ Ref: ADR 0007
    [ ] 4_requests_en_1_minuto_retorna_429()
        └─ Ref: ADR 0009
    [ ] honeypot_con_valor_retorna_200_silencioso()
        └─ Ref: ADR 0029
```

---

## L.3 — Layout + estructura de la landing

> **Referencia:** ADR 0022 (Frontend), ADR 0023 (i18n), docs/02-STACK.md L368-400, docs/03-STRUCTURE.md L459-462

```
[ ] apps/web/src/layouts/LandingLayout.astro:
    └─ Ref: docs/03-STRUCTURE.md L459-462
    [ ] Sin sidebar, sin verificación de sesión
        └─ Ref: ADR 0022 — landing es pública
    [ ] <html lang="es"> con data-theme
        └─ Ref: ADR 0023, docs/02-STACK.md L392-400
    [ ] <title>{props.title}</title>
    [ ] meta description, canonical
    [ ] Open Graph: og:title, og:description, og:image
    [ ] Twitter Cards: twitter:card, twitter:title
    [ ] Favicon + Apple touch icons
    [ ] Consent banner (carga analytics con opt-in)
    [ ] @astrojs/sitemap generado automáticamente
        └─ Ref: docs/02-STACK.md L381

[ ] apps/web/src/pages/index.astro:
    └─ Ref: docs/03-STRUCTURE.md L459
    [ ] LandingLayout con title y description
    [ ] Importa todas las secciones
    [ ] HTML puro desde servidor — cero JS para ver el contenido
        └─ Ref: ADR 0022 — SSR sin hidratación innecesaria

[ ] Estructura de archivos:
    └─ Ref: docs/03-STRUCTURE.md L543-548
    [ ] src/components/landing/Hero.astro
    [ ] src/components/landing/Problema.astro
    [ ] src/components/landing/Solucion.astro
    [ ] src/components/landing/Features.astro
    [ ] src/components/landing/SocialProof.astro
    [ ] src/components/landing/CallToAction.astro
    [ ] src/components/landing/Footer.astro
    [ ] src/components/landing/LeadForm.svelte  (único con interactividad)
        └─ Ref: ADR 0022 — Svelte para interactividad
```

---

## L.4 — Contenido y secciones

> **Referencia:** ADR 0022 (Frontend), ADR 0023 (i18n), docs/02-STACK.md L368-400, docs/03-STRUCTURE.md L543-548

```
[ ] Hero.astro:
    └─ Ref: docs/03-STRUCTURE.md L543
    [ ] Titular principal (H1) — propuesta de valor en <10 palabras
    [ ] Subtítulo — beneficio concreto
    [ ] CTA primario → ancla al formulario de leads
    [ ] Imagen o ilustración del producto

[ ] Problema.astro:
    └─ Ref: docs/03-STRUCTURE.md L544
    [ ] ¿Qué problema resuelve?
    [ ] Puntos de dolor del usuario objetivo

[ ] Solucion.astro:
    └─ Ref: docs/03-STRUCTURE.md L545
    [ ] Cómo el boilerplate resuelve el problema
    [ ] Screenshot o demo GIF del dashboard

[ ] Features.astro:
    └─ Ref: docs/03-STRUCTURE.md L546
    [ ] Beneficios principales con iconos SVG:
        [ ] Rust + arquitectura hexagonal → mantenible
            └─ Ref: ADR 0001
        [ ] RBAC completo desde el día uno
            └─ Ref: ADR 0006
        [ ] Deploy en $5/mes con zero-downtime
            └─ Ref: ADR 0014, docs/02-STACK.md L360-368
        [ ] Auth segura con PASETO (sin JWT)
            └─ Ref: ADR 0008
    [ ] Microinteracciones CSS (hover, focus)
    [ ] Sin JavaScript necesario — CSS puro
        └─ Ref: ADR 0022 — Astro SSR

[ ] SocialProof.astro:
    └─ Ref: docs/03-STRUCTURE.md L547
    [ ] Métricas, logos o casos de uso
    [ ] Stack conocido: Rust, Axum, SQLite, Svelte
        └─ Ref: docs/02-STACK.md L1-5

[ ] Footer.astro:
    └─ Ref: docs/03-STRUCTURE.md L548
    [ ] Políticas de privacidad + términos
    [ ] Contacto + redes
    [ ] Copyright + año actual

[ ] i18n con Paraglide (ADR 0023):
    └─ Ref: ADR 0023, docs/02-STACK.md L392-400
    [ ] Todos los textos de la landing en messages/es.json
        └─ Ref: docs/03-STRUCTURE.md L527-529
    [ ] Versión en inglés en messages/en.json
        └─ Ref: docs/03-STRUCTURE.md L530
    [ ] Mismas funciones m.hero_title(), m.features_rbac(), etc.
```

---

## L.5 — Formulario de leads (Back + Front)

> **Referencia:** ADR 0029 (Landing + Leads), ADR 0009 (Rate Limit), ADR 0007 (Errores), ADR 0018 (Jobs), docs/02-STACK.md L386-389, docs/03-STRUCTURE.md L549

```
[ ] apps/web/src/components/landing/LeadForm.svelte:
    └─ Ref: docs/03-STRUCTURE.md L549

    [ ] Campos:
        [ ] email (required)
        [ ] name (optional)
        [ ] honeypot: campo oculto con display:none  ← bots lo rellenan
            └─ Ref: ADR 0029 — anti-spam

    [ ] ArkType validation:
        └─ Ref: docs/02-STACK.md L389
        [ ] email: 'string.email'
        [ ] name: 'string?' (opcional)
        [ ] Errores en tiempo real (no esperar el submit)

    [ ] Lógica anti-spam:
        └─ Ref: ADR 0029
        [ ] Si honeypot tiene valor → retornar éxito visual sin enviar
        [ ] Rate limit 3 req/min en el backend (ADR 0009)
            └─ Ref: docs/02-STACK.md L143

    [ ] Estados del formulario:
        [ ] idle: formulario normal
        [ ] loading: botón deshabilitado + spinner
        [ ] success: "¡Gracias! Te avisaremos cuando lancemos." (sin reload)
        [ ] error: mensaje claro del error
            └─ Ref: ADR 0007

    [ ] TanStack mutation → POST /api/v1/leads:
        └─ Ref: docs/02-STACK.md L386
        [ ] onSuccess → status = 'success'
        [ ] onError 400 → "Email inválido"
            └─ Ref: ADR 0007
        [ ] onError 429 → "Demasiados intentos. Espera un momento."
            └─ Ref: ADR 0009
        [ ] onError 5xx → "Algo falló. Intenta de nuevo."
            └─ Ref: ADR 0007

    [ ] UX:
        [ ] placeholder="tu@email.com"
        [ ] Botón: "Notificarme del lanzamiento"
        [ ] Touch targets > 44px (mobile-first)
        [ ] No recargar la página en ningún estado
            └─ Ref: ADR 0022 — SPA behavior
```

---

## L.6 — SEO + performance + analytics

> **Referencia:** ADR 0022 (Frontend), ADR 0023 (i18n), ADR 0014 (Deploy), docs/02-STACK.md L368-400, docs/03-STRUCTURE.md L459-462

```
[ ] Meta tags SEO en LandingLayout.astro:
    └─ Ref: docs/03-STRUCTURE.md L459-462
    [ ] title único y descriptivo (<60 chars)
    [ ] description relevante (<160 chars)
    [ ] canonical URL

[ ] Open Graph:
    [ ] og:title, og:description, og:image (1200×630px)
    [ ] og:type = "website"

[ ] Sitemap:
    [ ] @astrojs/sitemap integrado
        └─ Ref: docs/02-STACK.md L381
    [ ] pages/sitemap.xml.ts configurado

[ ] Core Web Vitals — objetivos:
    └─ Ref: ADR 0022
    [ ] LCP < 2.5s (HTML desde servidor sin JS bloqueante)
    [ ] CLS < 0.1 (dimensiones explícitas en imágenes)
    [ ] INP < 100ms (Svelte 5 sin runtime pesado)
        └─ Ref: docs/02-STACK.md L375 — Svelte 5 Runes

[ ] Analytics (sin cookies por defecto):
    [ ] Matomo self-hosted O Plausible
    [ ] Carga condicional con opt-in en consent banner

[ ] Cache de assets en Caddy:
    └─ Ref: ADR 0014, docs/02-STACK.md L360-368
    [ ] @static path /assets/* /images/* /fonts/*
    [ ] Cache-Control: "public, max-age=31536000, immutable"

[ ] Verificar con Lighthouse:
    [ ] Performance > 90
    [ ] Accessibility > 90
    [ ] SEO > 95
```

---

## L.7 — Tests + deploy

> **Referencia:** ADR 0010 (Testing), ADR 0009 (Rate Limit), ADR 0029 (Landing), ADR 0014 (Deploy), docs/02-STACK.md L429-443, docs/03-STRUCTURE.md L275

```
[ ] Tests de endpoint (cargo nextest):
    └─ Ref: ADR 0010, docs/02-STACK.md L429-443
    [ ] POST /api/v1/leads con email válido → 200
    [ ] POST /api/v1/leads con email inválido → 400
        └─ Ref: ADR 0007
    [ ] POST /api/v1/leads con email duplicado → 200 (silencioso)
        └─ Ref: ADR 0029
    [ ] POST /api/v1/leads 4 veces en 1 minuto → 429 en el 4º
        └─ Ref: ADR 0009

[ ] Tests de formulario (Playwright opcional):
    └─ Ref: docs/02-STACK.md L440 — Playwright para E2E
    [ ] Flujo completo: email → submit → mensaje de éxito
    [ ] Honeypot relleno → mensaje de éxito sin enviar a backend
        └─ Ref: ADR 0029
    [ ] Email inválido → error en tiempo real
        └─ Ref: ADR 0007

[ ] Deploy:
    └─ Ref: ADR 0014, docs/02-STACK.md L360-368
    [ ] La landing se sirve desde el mismo Astro SSR del dashboard
        └─ Ref: ADR 0022
    [ ] No requiere configuración extra — mismo just deploy
        └─ Ref: docs/02-STACK.md L360-368 — Kamal
    [ ] Verificar que tudominio.com sirve la landing (no el dashboard)
    [ ] Verificar que /login muestra la página de login (no la landing)
        └─ Ref: docs/03-STRUCTURE.md L278 — routing
```

---

## Verificación final de la landing

```bash
# 1. La página carga rápido
curl -o /dev/null -w "%{time_total}" https://tudominio.com
# → < 0.5s

# 2. El formulario funciona
curl -X POST https://tudominio.com/api/v1/leads \
  -H "Content-Type: application/json" \
  -d '{"email":"visitor@test.com","name":"Visitante"}'
# → 200

# 3. El lead está en la DB
sqlite3 data/boilerplate.db "SELECT * FROM leads ORDER BY created_at DESC LIMIT 3"

# 4. Email de bienvenida enviado (ver logs en development)
# → "📧 [LogMailer] Email que se enviaría en producción"

# 5. Rate limit funciona
for i in 1 2 3 4; do
  curl -s -X POST /api/v1/leads -d '{"email":"spam@test.com"}' | jq '.error'
done
# → null, null, null, "too_many_requests"
```

---

## Diagrama de Flujo de la Landing

```
┌─────────────────────────────────────────────────────────────────────────┐
│  L.1 — DOMINIO + DB (Lead)                                             │
│  ├─ crates/domain/src/entities/lead.rs (Lead, LeadSource)              │
│  ├─ crates/domain/src/ports/lead_repository.rs                         │
│  ├─ crates/database/src/repositories/sqlite_lead_repository.rs        │
│  ├─ Migration SQL: CREATE TABLE leads                                  │
│  └─ CaptureLeadUseCase (encola EmailJob:LeadWelcome)                   │
│     └─ Ref: ADR 0029, 0006, 0018, 0019                                │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  L.2 — ENDPOINT BACKEND                                                │
│  ├─ Handler: POST /api/v1/leads (público, sin auth)                   │
│  ├─ Rate limit: 3 req/min (ADR 0009)                                   │
│  ├─ utoipa::path para OpenAPI                                          │
│  └─ Response: 200 (silencioso si duplicado), 400, 429                  │
│     └─ Ref: ADR 0029, 0009, 0021                                       │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  L.3 — LAYOUT + ESTRUCTURA                                             │
│  ├─ LandingLayout.astro (sin sidebar, sin auth)                        │
│  ├─ pages/index.astro (HTML puro SSR)                                  │
│  ├─ Componentes Astro: Hero, Problema, Solucion, Features               │
│  └─ LeadForm.svelte (único con interactividad)                          │
│     └─ Ref: ADR 0022, 0023                                            │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  L.4 — CONTENIDO Y SECCIONES                                           │
│  ├─ Hero.astro (propuesta de valor + CTA)                              │
│  ├─ Problema.astro (pain points)                                       │
│  ├─ Solucion.astro (screenshot dashboard)                              │
│  ├─ Features.astro (Rust + Hexagonal, RBAC, PASETO, Deploy $5)         │
│  ├─ SocialProof.astro (métricas, stack)                                │
│  └─ Footer.astro (legal, contacto)                                     │
│     └─ Ref: ADR 0022, 0023, 0001, 0006, 0008, 0014                   │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  L.5 — FORMULARIO DE LEADS                                             │
│  ├─ LeadForm.svelte (email, name, honeypot)                            │
│  ├─ ArkType validation en tiempo real                                    │
│  ├─ Anti-spam: honeypot oculto (bots → éxito falso)                    │
│  ├─ TanStack mutation → POST /api/v1/leads                               │
│  └─ Estados: idle → loading → success/error (sin reload)              │
│     └─ Ref: ADR 0029, 0009, 0007, 0018, 0022                          │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  L.6 — SEO + PERFORMANCE                                               │
│  ├─ Meta tags SEO (title, description, canonical)                        │
│  ├─ Open Graph (og:title, og:description, og:image)                    │
│  ├─ @astrojs/sitemap                                                   │
│  ├─ Core Web Vitals (LCP < 2.5s, CLS < 0.1, INP < 100ms)               │
│  ├─ Analytics (Matomo/Plausible con opt-in)                            │
│  └─ Cache Caddy (max-age=31536000)                                     │
│     └─ Ref: ADR 0022, 0023, 0014                                      │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  L.7 — TESTS + DEPLOY                                                  │
│  ├─ Tests endpoint: cargo nextest (200, 400 silencioso, 429)            │
│  ├─ Tests E2E: Playwright (formulario, honeypot)                       │
│  └─ Deploy: Kamal + Caddy (mismo que dashboard)                        │
│     └─ Ref: ADR 0010, 0009, 0029, 0014                                │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta/Librería | URL | Útil para |
|----------------------|-----|-----------|
| **Astro** | https://docs.astro.build | SSR, layouts, islands architecture |
| **Svelte 5 Runes** | https://svelte.dev/docs/svelte/what-are-runes | $state, reactivity |
| **Tailwind CSS v4** | https://tailwindcss.com/docs | Utility-first CSS |
| **ArkType** | https://arktype.io | Runtime validation |
| **TanStack Query** | https://tanstack.com/query/latest | Mutations, loading states |
| **Paraglide JS** | https://inlang.com/m/gerre34r/library-inlang-paraglideJs | i18n type-safe |
| **utoipa** | https://docs.rs/utoipa/latest | OpenAPI generation |
| **tower-governor** | https://docs.rs/tower-governor/latest | Rate limiting |
| **Resend** | https://resend.com/docs | Email API |
| **Playwright** | https://playwright.dev | E2E testing |
| **Caddy** | https://caddyserver.com/docs | Reverse proxy, caching |
| **Lighthouse** | https://developer.chrome.com/docs/lighthouse | Performance audit |

---

## Troubleshooting — Landing Page

### L.1 — Dominio + DB

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Migración falla | leads.email UNIQUE constraint | Verificar no hay emails duplicados previos — Ref: ADR 0029 |
| Lead no se guarda | Insert sin OR IGNORE | Usar `INSERT OR IGNORE` para idempotencia — Ref: ADR 0029 |
| EmailJob no encola | Queue no configurada | Verificar `storage.setup()` en worker — Ref: ADR 0018 |

### L.2 — Endpoint Backend

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| 401 en /api/v1/leads | auth_middleware aplicado | Quitar middleware de esta ruta (es pública) — Ref: ADR 0008 |
| 429 no retorna Retry-After | GovernorLayer sin configurar header | Añadir `Retry-After` en error handler — Ref: ADR 0009 |
| Lead duplicado retorna 409 | No se usa INSERT OR IGNORE | Cambiar a `INSERT OR IGNORE` en repository — Ref: ADR 0029 |
| OpenAPI no muestra endpoint | Sin `#[utoipa::path]` | Añadir macro al handler — Ref: ADR 0021 |

### L.3 — Layout + Estructura

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| LandingLayout carga sidebar | Copiado de DashboardLayout | Crear layout limpio sin sidebar — Ref: docs/03-STRUCTURE.md L459-462 |
| HTML tiene hidratación innecesaria | client:load en componentes Astro | Astro SSR no necesita hidratación — Ref: ADR 0022 |
| i18n no funciona | Paraglide no configurado | Revisar `astro.config.mjs` — Ref: ADR 0023 |

### L.4 — Contenido

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Textos no traducen | Falta en messages/en.json | Copiar claves de es.json — Ref: docs/03-STRUCTURE.md L527-530 |
| OG image no aparece en redes | og:image URL inválida o no absoluta | Usar URL absoluta https://... — Ref: ADR 0022 |
| Features sin iconos | SVG mal importados | Usar `import` o inline SVG — Ref: docs/03-STRUCTURE.md L546 |

### L.5 — Formulario

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Honeypot visible | display:none no aplicado | Usar Tailwind `hidden` o CSS inline — Ref: ADR 0029 |
| Submit recarga página | form sin preventDefault | `e.preventDefault()` en onSubmit — Ref: ADR 0022 |
| Errores no aparecen | ArkType no configurado | `type({ email: 'string.email' })` — Ref: docs/02-STACK.md L389 |
| Rate limit 429 no muestra mensaje | Frontend no maneja 429 | Añadir case 429 en onError — Ref: ADR 0009 |
| Bots envían spam | Honeypot no detectado | Verificar campo tiene valor antes de enviar — Ref: ADR 0029 |

### L.6 — SEO + Performance

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Lighthouse Performance < 90 | JS bloqueante o imágenes grandes | Partytown para scripts externos — Ref: ADR 0022 |
| CLS alto | Imágenes sin dimensiones | Añadir `width` y `height` — Ref: ADR 0022 |
| Sitemap no generado | @astrojs/sitemap no instalado | `pnpm add -D @astrojs/sitemap` — Ref: docs/02-STACK.md L381 |
| Analytics carga sin consent | Consent banner no implementado | Cargar solo tras opt-in — Ref: ADR 0022 |
| Cache no funciona | Caddy config sin @static | Revisar `Caddyfile` — Ref: ADR 0014 |

### L.7 — Tests + Deploy

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Test duplicado falla | Espera 409 pero retorna 200 | Test debe esperar 200 silencioso — Ref: ADR 0029 |
| Rate limit test inconsistente | Timing dependiente de máquina | Usar `tokio::time::pause()` — Ref: ADR 0009 |
| Landing sirve dashboard en / | Router mal configurado | Revisar orden de rutas en router.rs — Ref: docs/03-STRUCTURE.md L278 |
| /login muestra landing | Fallback de Astro | Revisar `pages/login.astro` existe — Ref: docs/03-STRUCTURE.md L478-480 |

---

**Nota:** Si un error persiste, revisar los ADRs 0029 (Landing), 0009 (Rate Limit), 0022 (Frontend) que son los más relevantes para esta fase.
