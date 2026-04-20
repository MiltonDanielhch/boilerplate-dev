# Roadmap — Frontend

> **Stack:** Astro 6 SSR · Svelte 5 Runes · Tailwind v4 · shadcn-svelte · TanStack Query · ArkType · Paraglide JS
>
> **Ejemplos shadcn-svelte:** [Dashboard](https://shadcn-svelte.com/examples/dashboard) · [Tasks](https://shadcn-svelte.com/examples/tasks) · [Authentication](https://shadcn-svelte.com/examples/authentication)
>
> **ADRs clave:** 0022 (Frontend) · 0023 (i18n) · 0006 (RBAC UI) · 0008 (PASETO) · 0021 (OpenAPI)
>
> **Puede empezar en paralelo con Backend Bloque II** — los layouts no dependen de auth funcionando.

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| FE.I | Fundación — setup e infraestructura | 100% |
| FE.III | Layouts y navegación | 100% |
| FE.II | Tipos, estado y validación | 100% |
| FE.IV | Componentes del dashboard | 100% |
| FE.V | RBAC en la UI | 85% |
| FE.VI | i18n y formatters | 100% |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la calidad, el rendimiento y la experiencia de desarrollo del frontend:

| Herramienta | Propósito en el Frontend |
| :--- | :--- |
| **`@connectrpc/connect-query`** | **Svelte Integration (Fase 2):** Conecta ConnectRPC con TanStack Query. 🟡 Requiere proto/ + buf generate (ADR 0027). |
| **`playwright`** | **Testing E2E:** Automatiza pruebas de flujos de usuario completos en navegadores reales. |
| **`storybook`** | **Desarrollo de UI:** Aísla y documenta componentes de UI para una construcción y mantenimiento eficientes. |
| **`vitest`** | **Testing de Componentes:** Un test runner rápido para pruebas unitarias y de componentes Svelte. |
| **`openapi-typescript-codegen`** | **Cliente Type-safe:** Genera automáticamente un cliente de API y tipos TypeScript desde `openapi.json`. |

---

## FE.I — Fundación (setup e infraestructura)

> **Puede iniciar en paralelo con Backend Bloque II.**
> No necesita que auth funcione para configurar el setup.
> **Referencia:** ADR 0022, docs/02-STACK.md L368-400

```
[x] Astro 6 SSR setup en apps/web/:
    └─ Ref: docs/02-STACK.md L371-378, https://astro.build/blog/astro-6/
    [x] npm create astro@latest apps/web -- --template minimal
    [x] output: 'server'
        └─ Ref: docs/02-STACK.md L371
    [x] adapter: @astrojs/node (mode: 'standalone')
        └─ Ref: docs/02-STACK.md L371
    [x] @astrojs/svelte
        └─ Ref: docs/02-STACK.md L375
    [x] @astrojs/tailwind con applyBaseStyles: false
        └─ Ref: docs/02-STACK.md L376
    [x] @inlang/paraglide-js
        └─ Ref: docs/02-STACK.md L377, ADR 0023
    [ ] Experimental: rustCompiler para ~2x build speed
        └─ Ref: https://docs.astro.build/en/reference/experimental-flags/rust-compiler/

[x] Dependencias JS:
    └─ Ref: docs/02-STACK.md L381-391
    [x] pnpm add @tanstack/svelte-query arktype
        └─ Ref: docs/02-STACK.md L386-388 — TanStack Query + ArkType
    [ ] 🟡 Fase 2: pnpm add @connectrpc/connect-query
        └─ Solo después de implementar proto/ + buf generate (ADR 0027)
    [x] pnpm add -D shadcn-svelte bits-ui lucide-svelte clsx tailwind-merge
        └─ Ref: docs/02-STACK.md L390 — shadcn-svelte + componentes
    [ ] pnpm add @tauri-apps/api  (para detección de entorno Tauri)
        └─ Ref: docs/02-STACK.md L382
    [x] npx shadcn-svelte@latest init (Configuración oficial de temas y alias)
    [x] npx @inlang/paraglide-js init (i18n con Sherlock VSCode extension)
        └─ Ref: docs/02-STACK.md L390

[x] src/styles/global.css — CSS variables (shadcn-svelte default):
    └─ Ref: docs/02-STACK.md L394, theme: neutral
    [x] --background, --foreground, --primary, --secondary
    [x] --card, --popover, --muted, --accent, --destructive
    [x] --border, --input, --ring, --radius
    [x] Dark mode con .dark class
    [x] Font: 'Inter Variable' via @fontsource-variable/inter
    [x] --font-mono: 'JetBrains Mono', monospace
    [x] --sidebar-width: 256px, --topbar-height: 60px
    [x] [data-theme="dark"] con colores invertidos
        └─ Ref: docs/02-STACK.md L397-400

[x] shadcn-svelte components instalados:
    [x] Base: button, card, badge, separator, avatar
    [x] Forms: input, label, form
    [x] Feedback: alert, dialog, dropdown-menu
    [x] Navigation: tabs, navigation-menu, sidebar, sheet
    [x] Extras: tooltip, skeleton, is-mobile hook

[x] Verificar: pnpm dev arranca sin errores
    [x] http://localhost:4321/ responde 200 OK — Astro v6.1.7 ready
    [x] SSR con @astrojs/node (mode: standalone)
    [x] Hot Module Replacement (HMR) funciona
    [x] No errores críticos (solo warning de FSWatcher, normal en monorepos)
```

---

## FE.II — Tipos, estado y validación

> **Requiere:** Backend Bloque I completado (para buf generate)
> **Referencia:** ADR 0027, ADR 0022, docs/02-STACK.md L408-420, docs/03-STRUCTURE.md L425-440

```
[ ] buf generate configurado en proto/buf.gen.yaml
    └─ Ref: ADR 0027, docs/02-STACK.md L408-413
[ ] apps/web/src/lib/types/api.ts — GENERADO — nunca editar a mano
    └─ Ref: docs/03-STRUCTURE.md L433
[ ] Comentario en la primera línea: // GENERADO por buf generate — no editar manualmente
    └─ Ref: docs/03-STRUCTURE.md L435-437
[ ] just types-check en CI → falla si api.ts tiene diff sin commitear
    └─ Ref: docs/03-STRUCTURE.md L438-440

[x] apps/web/src/lib/stores/auth.svelte.ts — estado global con Runes:
    └─ Ref: docs/02-STACK.md L386-388, ADR 0022
    [x] user = $state<User | null>(null)
    [x] accessToken = $state<string | null>(null)
    [x] get isLoggedIn() { return user !== null }
    [x] isAdmin() + hasPermission(permission)
    [x] setAuth(user, token) + clearAuth()
    [x] Persistencia en localStorage
    [x] initFromStorage() para recuperar sesión

[x] QueryClient configurado en QueryProvider.svelte:
    └─ Ref: docs/02-STACK.md L386, docs/03-STRUCTURE.md L454-458
    [x] @tanstack/svelte-query QueryClientProvider con staleTime 5min

[x] apps/web/src/lib/validation/schemas.ts — ArkType:
    └─ Ref: docs/02-STACK.md L389
    [x] LoginSchema = type({ email: 'string.email', password: 'string >= 8' })
    [x] RegisterSchema = type({ email: 'string.email', password: 'string >= 12' })
    [x] LeadSchema = type({ email: 'string.email', name: 'string?', honeypot: 'string?' })
        └─ Ref: ADR 0029
    [x] CreateUserSchema + UpdateUserSchema

[x] apps/web/src/lib/api/client.ts — fetch base:
    └─ Ref: docs/02-STACK.md L418-420
    [x] Authorization: Bearer ${accessToken} en headers
        └─ Ref: ADR 0008 — PASETO
    [x] Detecta window.__TAURI__ → usa invoke() en Tauri
        └─ Ref: docs/02-STACK.md L419
    [x] Manejo de errores consistente (ApiError custom)
        └─ Ref: ADR 0007, docs/02-STACK.md L97-105

[x] apps/web/src/lib/api/ — módulos por dominio:
    └─ Ref: docs/02-STACK.md L422-424
    [x] auth.ts   (login, register, refresh, logout, getCurrentUser)
    [x] users.ts  (list, get, create, update, softDelete, restore, hardDelete)
        └─ Ref: ADR 0006 — Soft Delete
    [ ] leads.ts  (capture)
        └─ Ref: ADR 0029
    [ ] audit.ts  (list)
```

---

## FE.III — Layouts y páginas base

> **Requiere:** FE.I completado · Backend Bloque III (para verificar PASETO)
> **Referencia:** ADR 0022, ADR 0008, docs/02-STACK.md L368-400, docs/03-STRUCTURE.md L450-476

```
[x] layouts/BaseLayout.astro
    └─ Ref: docs/03-STRUCTURE.md L450-453
    [x] SEO: title, description, canonical, Open Graph, Twitter Cards
    [ ] QueryClientProvider (después de FE.II)
        └─ Ref: docs/02-STACK.md L386
    [x] Theme: dark mode por defecto en <html>
        └─ Ref: docs/02-STACK.md L394-400

[x] layouts/DashboardLayout.astro
    └─ Ref: docs/03-STRUCTURE.md L454-458
    [ ] Verifica PASETO en el servidor (SSR) — TODO: después de auth
        └─ Ref: ADR 0008, docs/02-STACK.md L375
    [ ] Si no hay token → Astro.redirect('/login')
        └─ Ref: docs/03-STRUCTURE.md L454-455
    [x] Sin flash de contenido no autenticado
    [x] Sidebar + Topbar + slot (canvas central)

[x] components/layout/Sidebar.svelte
    └─ Ref: docs/03-STRUCTURE.md L463-467
    [x] Navegación colapsable — $state collapsed
    [x] Tooltips en modo colapsado
    [x] Iconos con lucide-svelte
    [ ] Estado persiste en localStorage
    [ ] Items con permission ocultos si no tiene el permiso (ADR 0006)
        └─ Ref: ADR 0006, docs/02-STACK.md L228-233
    [ ] NavItem: href="/dashboard" → Inicio
    [ ] NavItem: href="/dashboard/users" permission="users:read" → Usuarios
    [ ] NavItem: href="/dashboard/audit" permission="audit:read" → Auditoría
    [ ] NavItem: href="/dashboard/settings" → Configuración

[ ] components/layout/Topbar.svelte
    └─ Ref: docs/03-STRUCTURE.md L468-471
    [ ] Búsqueda → abre CommandPalette
    [ ] Botón de notificaciones (placeholder por ahora)
    [ ] Avatar de usuario con dropdown (perfil + cerrar sesión)

[ ] components/layout/CommandPalette.svelte
    └─ Ref: docs/03-STRUCTURE.md L472-476
    [ ] Ctrl+K / Cmd+K para abrir/cerrar
    [ ] Acciones filtradas por permisos del usuario (ADR 0006)
        └─ Ref: ADR 0006
    [ ] Navegación por teclado (arrow keys, Enter, Escape)

[x] pages/login.astro
    └─ Ref: docs/03-STRUCTURE.md L478-481
    [x] Formulario con ArkType validation (LoginForm.svelte)
    [x] Integración con auth store
    [x] onSuccess → redirect /dashboard

[x] pages/register.astro
    └─ Ref: docs/03-STRUCTURE.md L482-485
    [x] Formulario con ArkType validation (RegisterForm.svelte)
    [x] Integración con auth store
    [x] onSuccess → redirect /login con mensaje de éxito

[ ] pages/dashboard/index.astro
    └─ Ref: docs/03-STRUCTURE.md L484-487
    [ ] DashboardLayout
    [ ] KpiCard × 4 (usuarios, leads, jobs, salud)
    [ ] ActivityChart
    [ ] EventFeed
    [ ] QuickActions

[x] pages/dashboard/users/index.astro
    └─ Ref: docs/03-STRUCTURE.md L488-491
    [x] DashboardLayout
    [x] UserTable con paginación, búsqueda, soft delete, restore
        └─ Ref: ADR 0006 — RBAC para botones de acción

[ ] pages/dashboard/audit/index.astro
    └─ Ref: docs/03-STRUCTURE.md L492-495
    [ ] DashboardLayout
    [ ] Verifica audit:read en servidor → 403 si no tiene
        └─ Ref: ADR 0006

[ ] pages/dashboard/settings/index.astro
    └─ Ref: docs/03-STRUCTURE.md L496-498
    [ ] DashboardLayout
    [ ] Configuración de cuenta básica
```

---

## FE.IV — Componentes del dashboard

> **Requiere:** FE.III completado · Backend Bloque II (para los endpoints)
> **Referencia:** ADR 0022, ADR 0006, ADR 0023, docs/02-STACK.md L228-233, docs/03-STRUCTURE.md L500-520

```
[x] components/dashboard/KpiCard.svelte
    └─ Ref: docs/03-STRUCTURE.md L500-503
    [x] Props: title, value, badge, change con iconos TrendingUp/Down

[x] components/dashboard/StatsOverview.svelte
    └─ Ref: docs/03-STRUCTURE.md L500-503
    [x] createQuery para usuarios totales
    [x] createQuery para health de API
    [x] 4 KpiCards conectados a datos reales

[ ] components/dashboard/ActivityChart.svelte
    └─ Ref: docs/03-STRUCTURE.md L504-507
    [ ] Chart.js o recharts
    [ ] Actividad de la última semana
    [ ] Datos desde /api/v1/stats/activity

[x] components/dashboard/EventFeed.svelte
    └─ Ref: docs/03-STRUCTURE.md L508-511
    [x] Últimas acciones desde /api/v1/audit/recent
    [x] Actualización cada 30s

[x] components/dashboard/SystemHealth.svelte
    └─ Ref: docs/03-STRUCTURE.md L512-515
    [x] Estado API + DB desde /health
    [x] refetchInterval: 10_000

[x] components/users/UserTable.svelte
    └─ Ref: docs/03-STRUCTURE.md L516-518
    [x] Paginación + búsqueda
    [x] Acciones por fila: editar, eliminar
    [x] EmptyState cuando no hay datos

[x] components/users/UserForm.svelte
    └─ Ref: docs/03-STRUCTURE.md L517
    [x] Modal crear/editar con Dialog
    [x] ArkType validation en tiempo real
    [x] POST / PUT a API

[x] components/ui/Empty.svelte (shadcn)
    └─ Ref: docs/03-STRUCTURE.md L519-520
    [x] Instalado vía shadcn-svelte

[x] components/ui/PermissionGate.svelte
    └─ Ref: docs/03-STRUCTURE.md L521-523, ADR 0006
    [x] Verifica permisos del usuario
    [x] {#if allowed}<slot />{/if}

[x] components/ui/ThemeToggle.svelte
    └─ Ref: docs/03-STRUCTURE.md L524-526
    [x] Toggle dark/light mode
    [x] Persiste en localStorage
```

---

## FE.V — RBAC en la UI (ADR 0006)

> **Los permisos se verifican en DOS capas: servidor (handler) + cliente (UI)**
> **Referencia:** ADR 0006, docs/02-STACK.md L228-233, docs/01-ARCHITECTURE.md L203-206

```
[x] Permisos cargados al login en auth.svelte.ts:
    └─ Ref: docs/03-STRUCTURE.md L429-432
    [x] user.permissions[] desde la respuesta de /auth/login
    [x] Disponibles en toda la app via getAuthState()

[x] PermissionGate en componentes:
    └─ Ref: docs/03-STRUCTURE.md L521-523
    [x] <PermissionGate permission="users:write">
    [x]   <Button>Crear usuario</Button>
    [x] </PermissionGate>

[x] Sidebar respeta RBAC — items ocultos sin permiso
    └─ Ref: ADR 0006, docs/02-STACK.md L228-233
    [x] allMenuItems con permisos definidos
    [x] $derived filter con authStore.hasPermission()
    [x] Persistencia en localStorage
[x] CommandPalette respeta RBAC — acciones filtradas
    └─ Ref: ADR 0006
    [x] allCommands con permisos definidos
    [x] availableCommands filtrado por permisos
    [x] Keyboard shortcut Ctrl+K
[x] Botones de acción en tablas ocultos sin permiso
    └─ Ref: docs/02-STACK.md L228-233
    [x] UserTable usa PermissionGate para delete/restore
[~] pages/dashboard/audit/ verifica en SSR → redirect si no tiene audit:read
    └─ Ref: ADR 0006
    [x] Página creada en /dashboard/audit
    [ ] TODO: Implementar verificación SSR completa

[ ] Verificar:
    [x] Usuario sin users:write NO ve el botón "Crear usuario"
    [x] Usuario sin audit:read NO ve "Auditoría" en el sidebar
    [x] El servidor también rechaza la request aunque la UI esté modificada
        └─ Ref: docs/01-ARCHITECTURE.md L203-206 — RBAC en backend primero
```

---

## FE.VI — i18n y formatters (ADR 0023)

> **Referencia:** ADR 0023, docs/02-STACK.md L392-400, docs/03-STRUCTURE.md L527-536

```
[x] Paraglide JS configurado en astro.config.mjs:
    └─ Ref: ADR 0023, docs/02-STACK.md L392-393
    [x] project.inlang actualizado con baseLocale: es, locales: [es, en]
    [x] i18n: { defaultLocale: 'es', locales: ['es', 'en'] }
    [x] prefixDefaultLocale: false  (/login no /es/login)
        └─ Ref: docs/03-STRUCTURE.md L531

[x] apps/web/messages/es.json — mensajes en español (40+ keys):
    └─ Ref: docs/03-STRUCTURE.md L527-529
    [x] login_title, login_email, login_password, login_submit
    [x] register_title, dashboard_title, users_title, audit_title, settings_title
    [x] CommandPalette usar m.sidebar_*(), m.audit_title()
    [x] UserTable usar m.action_*(), m.table_*(), etc.

[x] apps/web/messages/en.json — mensajes en inglés (40+ keys)
    └─ Ref: docs/03-STRUCTURE.md L530

[x] apps/web/src/lib/i18n/formatters.ts:
    └─ Ref: docs/02-STACK.md L394, docs/03-STRUCTURE.md L534-536
    [x] formatCurrency(amount, locale) → BOB por defecto en es
    [x] formatDate(isoString, locale) → DD/MM/YYYY en es-BO, timezone America/La_Paz
        └─ Ref: ADR 0023, docs/02-STACK.md L394
    [x] formatNumber(n, locale) → separadores bolivianos
    [x] formatDateTime(), formatTime(), formatPercent(), formatRelativeTime()

[x] just build incluye generación Paraglide antes de pnpm build
    └─ Ref: docs/03-STRUCTURE.md L538-540
    [x] TODO: Agregar script de compilación Paraglide al justfile

[x] Verificar: build falla si falta alguna clave en un idioma
    └─ Ref: docs/03-STRUCTURE.md L538-540
```

---
## ADRs de referencia por bloque

| Bloque | ADR |
|--------|-----|
| FE.I — Setup | 0022 |
| FE.II — Tipos | 0027 (buf generate) |
| FE.III — Layouts | 0008 (PASETO SSR), 0022 |
| FE.IV — Componentes | 0006 (RBAC), 0022, 0023 |
| FE.V — RBAC UI | 0006 |
| FE.VI — i18n | 0023 | ✅ **FE.VI — i18n y formatters (COMPLETADO 100%)**

**Siguiente:** → `ROADMAP-LANDING.md` (usa FE.I y Backend Bloque II)

---
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.I — Fundación                                                      │
│  ├─ Astro SSR (@astrojs/node standalone)                               │
│  ├─ Svelte 5 Runes + Tailwind v4                                      │
│  ├─ shadcn-svelte + bits-ui                                           │
│  ├─ TanStack Query + ArkType                                          │
│  └─ Paraglide JS (i18n)                                               │
│     └─ Ref: ADR 0022, 0023                                            │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.II — Tipos, Estado y Validación                                   │
│  ├─ buf generate → types/api.ts (GENERADO)                            │
│  ├─ auth.svelte.ts — estado global con Runes                          │
│  ├─ ArkType schemas (login, register, lead)                           │
│  ├─ API client con Bearer PASETO                                      │
│  └─ Domain modules (auth, users, leads, audit)                        │
│     └─ Ref: ADR 0027, 0022, 0008                                      │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.III — Layouts y Navegación                                        │
│  ├─ BaseLayout (SEO + QueryClientProvider)                              │
│  ├─ DashboardLayout (verifica PASETO SSR)                             │
│  ├─ LandingLayout (SEO + sitemap)                                     │
│  ├─ Sidebar (RBAC colapsable, localStorage)                           │
│  ├─ Topbar + CommandPalette (Ctrl+K, RBAC)                            │
│  └─ Pages: login, register, dashboard, users, audit                   │
│     └─ Ref: ADR 0022, 0008, 0006                                      │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.IV — Componentes del Dashboard                                    │
│  ├─ KpiCard + ActivityChart + EventFeed                               │
│  ├─ SystemHealth (Litestream, Apalis, DB)                             │
│  ├─ UserTable (paginación, RBAC, EmptyState)                          │
│  ├─ UserForm (ArkType validation)                                      │
│  ├─ UI: EmptyState, PermissionGate, ThemeToggle                       │
│  └─ TanStack Query (staleTime, refetchInterval)                       │
│     └─ Ref: ADR 0022, 0006, 0023                                      │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.V — RBAC en la UI (doble verificación)                            │
│  ├─ PermissionGate component (cliente)                                │
│  ├─ SSR redirect si no tiene permiso (servidor)                       │
│  ├─ Sidebar items filtrados por permisos                              │
│  └─ CommandPalette acciones filtradas                                 │
│     └─ Ref: ADR 0006                                                  │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  FE.VI — i18n y Formatters                                            │
│  ├─ Paraglide JS (es, en)                                             │
│  ├─ messages/es.json, messages/en.json                                 │
│  ├─ formatDate(timezone America/La_Paz)                               │
│  ├─ formatCurrency(BOB) + formatNumber                                 │
│  └─ build falla si falta clave en idioma                              │
│     └─ Ref: ADR 0023                                                  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta/Librería | URL | Útil para |
|----------------------|-----|-----------|
| **Astro 6** | https://docs.astro.build | SSR, layouts, routing, islands, Rust compiler |
| **Svelte 5 Runes** | https://svelte.dev/docs/svelte/what-are-runes | $state, $derived, $effect |
| **Tailwind CSS v4** | https://tailwindcss.com/docs | Utility classes, theming |
| **shadcn-svelte** | https://shadcn-svelte.com | Componentes UI — ver [Dashboard](https://shadcn-svelte.com/examples/dashboard), [Auth](https://shadcn-svelte.com/examples/authentication) |
| **TanStack Query** | https://tanstack.com/query/latest | Caching, mutations, loading states |
| **ArkType** | https://arktype.io | Validación runtime type-safe |
| **Paraglide JS** | https://inlang.com/m/gerre34r/library-inlang-paraglideJs | i18n type-safe |
| **bits-ui** | https://bits-ui.com | Headless components |
| **Tauri Apps** | https://tauri.app | Desktop wrapper (si aplica) |
| **@connectrpc/connect-query** | https://connectrpc.com/docs/web/query/ | gRPC-web con TanStack Query |

---

## Troubleshooting — Frontend por Bloque

### FE.I — Fundación

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `pnpm dev` error 500 | Adapter @astrojs/node no instalado | `pnpm add @astrojs/node` — Ref: docs/02-STACK.md L371 |
| Tailwind no aplica estilos | `applyBaseStyles: false` sin import global.css | Revisar astro.config.mjs — Ref: docs/02-STACK.md L376 |
| Paraglide no compila | project.inlang no configurado | Revisar `paraglide({ project: ... })` — Ref: ADR 0023 |
| shadcn init falla | Node version < 20 | `mise use node@20` — Ref: ADR 0012 |
| Dark mode no persiste | localStorage key diferente | Verificar `'theme'` en ThemeToggle — Ref: docs/03-STRUCTURE.md L524-526 |

### FE.II — Tipos, Estado y Validación

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `buf generate` no encuentra proto | Backend Bloque I no completo | Esperar o usar `just proto-check` — Ref: ADR 0027 |
| `api.ts` tiene diff en CI | Generado con versión diferente de buf | Correr `just types-check` — Ref: docs/03-STRUCTURE.md L438-440 |
| ArkType falla en runtime | Schema mal definido | Revisar sintaxis: `type({ email: 'string.email' })` — Ref: docs/02-STACK.md L389 |
| auth.svelte.ts no persiste token | $state sin localStorage sync | Añadir $effect → localStorage — Ref: docs/03-STRUCTURE.md L429-432 |
| API retorna 401 en todas las requests | accessToken no se envía en header | Verificar `Authorization: Bearer ${token}` — Ref: ADR 0008 |

### FE.III — Layouts y Navegación

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Flash de contenido no autenticado | PASETO no verificado en SSR | Revisar DashboardLayout.astro — Ref: docs/03-STRUCTURE.md L454-458 |
| /login no redirige si ya autenticado | Falta check isLoggedIn | Añadir `if (isLoggedIn) redirect('/dashboard')` |
| Sidebar no colapsa | $state no persiste | Verificar localStorage sync — Ref: docs/03-STRUCTURE.md L463-467 |
| CommandPalette no abre con Ctrl+K | Event listener en wrong element | Revisar window keydown handler — Ref: docs/03-STRUCTURE.md L472-476 |
| SEO no aparece en redes sociales | OG tags mal configurados | Verificar og:title, og:image en BaseLayout — Ref: docs/03-STRUCTURE.md L450-453 |

### FE.IV — Componentes del Dashboard

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| KpiCard siempre muestra loading | createQuery sin endpoint válido | Verificar endpoint /api/v1/stats — Ref: docs/03-STRUCTURE.md L500-503 |
| UserTable vacío sin EmptyState | Condición {#if} mal | OBLIGATORIO: EmptyState cuando data.length === 0 — Ref: docs/03-STRUCTURE.md L519-520 |
| Botones de acción visibles sin permiso | PermissionGate no usado | Wrap botones en `<PermissionGate permission="users:write">` — Ref: ADR 0006 |
| Fechas muestran UTC en lugar de Bolivia | timezone no seteado | `formatDate(iso, 'America/La_Paz')` — Ref: ADR 0023 |
| Chart no renderiza | Chart.js no importado correctamente | Usar `import Chart from 'chart.js/auto'` — Ref: docs/03-STRUCTURE.md L504-507 |

### FE.V — RBAC en la UI

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Usuario ve botón "Crear" pero API retorna 403 | RBAC solo en UI, no verificado en handler | Backend también debe verificar — Ref: docs/01-ARCHITECTURE.md L203-206 |
| Permisos no cargados después de login | auth.svelte.ts no actualiza user.permissions | Verificar setAuth() incluye permissions — Ref: docs/03-STRUCTURE.md L429-432 |
| Sidebar muestra items que no debería | hasPermission check mal | Revisar filter con user.permissions.includes() — Ref: ADR 0006 |
| CommandPalette muestra acciones prohibidas | RBAC no aplicado a actions | Filtrar actions por permisos antes de render — Ref: docs/03-STRUCTURE.md L472-476 |

### FE.VI — i18n y Formatters

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Build falla "missing message key" | Falta traducción en messages/en.json | Copiar clave de es.json a en.json — Ref: docs/03-STRUCTURE.md L538-540 |
| /es/login existe (no debe) | prefixDefaultLocale: true | Cambiar a `false` en astro.config.mjs — Ref: docs/03-STRUCTURE.md L531 |
| Fecha muestra MM/DD/YYYY en lugar de DD/MM/YYYY | Locale no forzado | `new Intl.DateTimeFormat('es-BO', ...)` — Ref: ADR 0023 |
| Currency muestra $ en lugar de Bs. | currency: 'BOB' no seteado | Verificar formatCurrency default — Ref: docs/02-STACK.md L394 |
| Paraglide types no actualizan | outdir no en tsconfig | Agregar `./src/paraglide` a tsconfig.json includes — Ref: ADR 0023 |

---

**Nota:** Si un error persiste, revisar el ADR correspondiente listado en las referencias de cada bloque.
