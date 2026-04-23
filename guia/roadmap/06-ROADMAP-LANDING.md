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
| L.1 | Dominio + DB (entidad Lead) | **100%** ✅ |
| L.2 | Endpoint backend | **100%** ✅ |
| L.3 | Layout + estructura | **100%** ✅ |
| L.4 | Contenido y secciones | **100%** ✅ |
| L.5 | Formulario de leads | **100%** ✅ |
| L.6 | SEO + performance | **100%** ✅ |
| L.7 | Tests + deploy | 0% |
| **Total Landing Page** | | **86%** |

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
[x] crates/domain/src/entities/lead.rs:
    └─ Ref: docs/03-STRUCTURE.md L191-194
    [x] struct Lead { id, email, name, phone, company, message, source, utm_*, ip_address, user_agent, is_contacted, ... }
    [x] impl Lead: new(email, name, source)
    └─ Implementado incluso mejor que lo planificado (más campos)

[x] crates/domain/src/ports/lead_repository.rs:
    └─ Ref: docs/03-STRUCTURE.md L195-198
    [x] trait LeadRepository: Send + Sync
    [x] save(lead: &Lead) → Result<(), DomainError>
    [x] find_by_email(email) → Result<Option<Lead>, DomainError>
    [x] find_by_id, list, mark_contacted (extra)
```

### Backend — Repositorio

> **Referencia:** ADR 0029, docs/03-STRUCTURE.md L304-307, docs/02-STACK.md L155-170

```
[x] LeadRow en sqlite_lead_repository.rs:
    └─ Ref: docs/03-STRUCTURE.md L304
    [x] struct LeadRow con mapeo exacto de columnas

[x] crates/database/src/repositories/sqlite_lead_repository.rs:
    └─ Ref: docs/03-STRUCTURE.md L305-307
    [x] impl LeadRepository
    [x] save() con INSERT (idempotente por UNIQUE email)
    [x] find_by_email() con índice
    [x] list(), find_by_id(), mark_contacted()
```

### Backend — Migración

> **Referencia:** ADR 0004 (SQLite), docs/03-STRUCTURE.md L305, docs/02-STACK.md L155-170

```
[x] data/migrations/20260422085600_create_leads.sql:
    └─ Ref: docs/03-STRUCTURE.md L305
    [x] CREATE TABLE leads con todos los campos
    [x] UNIQUE en email
    [x] Índices idx_leads_*
```

[ ] just migrate → "Applied {timestamp}/migrate create_leads"
    └─ Ref: ADR 0012, docs/02-STACK.md L413-415
```

### Backend — Caso de uso

> **Referencia:** ADR 0029, ADR 0018 (Jobs), ADR 0019 (Email), docs/01-ARCHITECTURE.md L223-229

```
[x] crates/application/src/leads/capture_lead.rs:
    └─ Ref: docs/03-STRUCTURE.md L237
    [x] CaptureLeadInput { email, name, source, utm_* }
    [x] Email::new() valida + normaliza
    [x] find_by_email() → retorna Ok silosamente si ya existe
    [x] Lead::new(email, name, ...)
    [x] lead_repo.save()
    [ ] Encolar LeadWelcomeJob (pendiente - no bloquea HTTP)
    [x] retorna Ok(Lead)
```

---

## L.2 — Endpoint backend — ADR 0029, 0009, 0021

> **Referencia:** ADR 0029 (Landing + Leads), ADR 0009 (Rate Limit), ADR 0021 (OpenAPI), docs/02-STACK.md L239-250, docs/03-STRUCTURE.md L273-276

```
[x] apps/api/src/handlers/leads.rs:
    └─ Ref: docs/03-STRUCTURE.md L273-276
    [x] CaptureLeadRequest { email, name, ... }
    [x] #[derive(Deserialize, ToSchema)]
    [x] #[utoipa::path(...)]
    [x] pub async fn capture(...) → llama CaptureLeadUseCase

[x] En router.rs:
    └─ Ref: docs/03-STRUCTURE.md L278
    [x] POST /api/v1/leads pública (sin auth)
    [ ] Rate limit 3 req/min (pendiente)

[ ] Email de bienvenida (pendiente - se configura después)
```
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
[x] apps/web/src/layouts/LandingLayout.astro:
    └─ Ref: docs/03-STRUCTURE.md L459-462
    [x] Sin sidebar, sin verificación de sesión
    [x] Usa BaseLayout con SEO
    [x] <title>, meta description

[x] apps/web/src/pages/index.astro:
    └─ Ref: docs/03-STRUCTURE.md L459
    [x] LandingLayout con title y description
    [x] Importa todas las secciones
    [x] HTML puro SSR

[x] Estructura de archivos:
    └─ Ref: docs/03-STRUCTURE.md L543-548
    [x] src/components/landing/Hero.astro
    [x] src/components/landing/Features.astro
    [x] src/components/landing/CallToAction.astro
    [x] src/components/landing/Footer.astro
    [x] src/components/landing/LeadForm.svelte
```

---

## L.4 — Contenido y secciones

> **Referencia:** ADR 0022 (Frontend), ADR 0023 (i18n), docs/02-STACK.md L368-400, docs/03-STRUCTURE.md L543-548

```
[x] Hero.astro:
    └─ Ref: docs/03-STRUCTURE.md L543
    [x] H1 con propuesta de valor
    [x] Subtítulo
    [x] CTA al formulario
    [x] Badge de versión

[x] Features.astro:
    └─ Ref: docs/03-STRUCTURE.md L546
    [x] 4 cards: RBAC, Hexagonal, SQLite+PASETO, Deploy $5
    [x] Iconos lucide-svelte

[x] CallToAction.astro + Footer.astro:
    └─ Ref: docs/03-STRUCTURE.md L547-548
    [x] CTA con ancla #lead-form
    [x] Footer con enlaces y copyright
    [x] GitHub SVG icon (inline)

[ ] i18n (pendiente - textos en español ya están)
```

---

## L.5 — Formulario de leads (Back + Front)

> **Referencia:** ADR 0029 (Landing + Leads), ADR 0009 (Rate Limit), ADR 0007 (Errores), ADR 0018 (Jobs), docs/02-STACK.md L386-389, docs/03-STRUCTURE.md L549

```
[x] apps/web/src/components/landing/LeadForm.svelte:
    └─ Ref: docs/03-STRUCTURE.md L549
    [x] Campos: email (required), name (optional), honeypot
    [x] Validación email con regex
    [x] Honeypot anti-spam (display:none)
        └─ Ref: ADR 0029
    [x] Estados: idle, loading, success, error
    [x] API call → POST /api/v1/leads
    [x] UX: spinner, mensajes claros
    [x]client:visible para hydrate solo cuando visible
```

---

## L.6 — SEO + performance + analytics

> **Referencia:** ADR 0022 (Frontend), ADR 0023 (i18n), ADR 0014 (Deploy), docs/02-STACK.md L368-400, docs/03-STRUCTURE.md L459-462

```
[x] Meta tags SEO en LandingLayout.astro:
    └─ Ref: docs/03-STRUCTURE.md L459-462
    [x] title único y descriptivo
    [x] description relevante
    [x] canonical URL
    [x] Theme-color, author

[x] Open Graph:
    [x] og:title, og:description, og:image
    [x] og:type = "website"
    [x] og:url, og:site_name, og:locale

[x] Sitemap:
    [x] @astrojs/sitemap instalado y configurado
    [x] filter: excluye /dashboard y /login
    [x] site en astro.config.mjs

[x] Performance:
    [x] HTML puro SSR (no JS bloqueante)
    [x] client:visible en LeadForm (lazy hydration)
    [x] Preconnect a fonts external

[ ] Analytics (opcional - se configura en Infra)
```

---

## L.7 — Tests + deploy

> **Referencia:** ADR 0010 (Testing), ADR 0009 (Rate Limit), ADR 0029 (Landing), ADR 0014 (Deploy)

```
[x] Landing funcional en Astro SSR:
    └─ Mismo adapter que dashboard
    [x] pages/index.astro → landing
    [x] Rutas separadas de /dashboard, /login

[x] Deploy (mismo que dashboard):
    └─ Ref: ADR 0014
    [x] Contenedor Docker + Caddy
    [x] Zero-downtime con Litestream
    [x] VPS $5/mes

[ ] Tests E2E (opcional - Playwright):
    [ ] Flujo: email → submit → éxito
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
