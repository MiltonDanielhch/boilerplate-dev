# Roadmap Master — boilerplate

> El mapa de todos los mapas. Cada bloque tiene su propio documento detallado.
> **Fuente de verdad:** este archivo + `TODO.md`

---

## Estado global

| Fase | Documento | Foco | Cuando empezar | Estado |
|------|-----------|------|----------------|--------|
| 0 | `ROADMAP-GENESIS.md` | Workspace + tooling | Día 1 | ⏳ |
| 1 | `ROADMAP-BACKEND.md` | Dominio + DB + Auth + API | Día 1-2 | ⏳ |
| 2 | `ROADMAP-FRONTEND.md` | Astro + Svelte + Dashboard | Paralelo con Backend Bloque II | ⏳ |
| 3 | `ROADMAP-AUTH-FULLSTACK.md` | Login/Registro back+front | Después de Backend III + Frontend III | ⏳ |
| 4 | `ROADMAP-LANDING.md` | Landing + captura de leads | Paralelo con Auth Fullstack | ⏳ |
| 5 | `ROADMAP-INFRA.md` | Deploy + Caddy + Kamal | MVP backend+frontend listo | ⏳ |
| — | **MVP EN PRODUCCIÓN** | — | — | — |
| 6 | `ROADMAP-TAURI-DESKTOP.md` | Desktop Tauri 2.0 | MVP web validado | ⏳ |
| 7 | `ROADMAP-MOBILE.md` | Mobile Tauri + KMP | Desktop validado | ⏳ |

---

## Orden de ejecución — el camino crítico

```
DÍA 1 ─── GÉNESIS
│         Workspace + crates + justfile + tooling
│         cargo check --workspace ✓
│
DÍA 2 ─── BACKEND I — Fundación 🔥 CRÍTICO
│         6 migraciones + dominio + repositorios
│         NO pasar al Bloque II sin las 6 migraciones pasando
│
DÍA 3 ─── BACKEND II + FRONTEND I (paralelo)
│         Axum setup + middleware ← Backend
│         Astro setup + Tailwind + Svelte 5 ← Frontend
│         (Frontend no depende del backend en esta fase)
│
DÍA 4 ─── BACKEND III + FRONTEND II (paralelo)
│         Auth + RBAC + Audit ← Backend 🔥 CRÍTICO
│         Tipos + auth store + ArkType ← Frontend
│
DÍA 5 ─── BACKEND IV + FRONTEND III (paralelo)
│         OpenAPI + Scalar ← Backend
│         Layouts + páginas base ← Frontend
│
DÍA 6 ─── AUTH FULLSTACK (back + front coordinados)
│         Registro, login, sesiones, RBAC en UI
│         Test E2E completo del flujo de auth
│
DÍA 7 ─── BACKEND V + LANDING (paralelo)
│         Jobs + Cache + Email ← Backend
│         Landing page + formulario de leads ← Landing
│
DÍA 8 ─── BACKEND VI + FRONTEND IV (paralelo)
│         Observabilidad ← Backend
│         Componentes del dashboard ← Frontend
│
DÍA 9 ─── INFRA — Deploy
│         Containerfile + Caddy + Kamal + Litestream
│         just deploy ✓ + kamal rollback ✓
│
──────────── MVP EN PRODUCCIÓN ────────────────────────────
│
DÍA 10+ ─ DESKTOP (solo si MVP web validado)
│         Tauri 2.0 + comandos Rust + build Win/Mac/Linux
│
DÍA 12+ ─ MOBILE (solo si Desktop validado)
          Tauri Mobile + PWA + KMP si necesario
```

---

## Qué puedes hacer en paralelo

| Tarea A | Tarea B | Se pueden hacer en paralelo? |
|---------|---------|------------------------------|
| Backend I — Migraciones | Frontend I — Setup Astro | ✅ Sí — no se tocan |
| Backend II — Axum | Frontend I — Setup Astro | ✅ Sí |
| Backend III — Auth | Frontend II — Tipos + store | ✅ Sí |
| Backend IV — OpenAPI | Frontend III — Layouts | ⚠️ Parcial — Requiere contrato `api.ts` |
| Backend V — Jobs | Landing — Estructura | ✅ Sí |
| Auth Fullstack | Landing | ✅ Sí (Landing no necesita auth) |
| Backend VI — Observabilidad | Frontend IV — Componentes | ✅ Sí |
| Auth Fullstack | Backend IV — OpenAPI | ✅ Sí |
| Deploy (Infra) | Desktop | ❌ No — Deploy debe completarse primero |
| Desktop | Mobile | ❌ No — Desktop debe estar validado primero |

---

## Entregables por fase

### Génesis
```
cargo check --workspace  → verde
cargo deny check         → verde
just --list              → muestra todos los comandos
grep "jsonwebtoken" .    → cero resultados
```

### Backend completo
```
POST /auth/register → 201
POST /auth/login → 200 + "v4.local.xxx" (PASETO, no JWT)
GET /api/v1/users (con token + permiso) → 200
GET /api/v1/users (sin token) → 401
GET /api/v1/users (sin permiso) → 403
curl http://localhost:8080/health → {"status":"ok"}
```

### Frontend completo
```
pnpm dev → arranca sin errores
/login → formulario funcional con validación
/dashboard → KPIs + tabla de usuarios
RBAC → botones ocultos sin permiso
Modo oscuro/claro funciona
```

### Auth Fullstack
```
Flujo E2E: register → login → dashboard → logout
Token empieza con "v4.local." (PASETO) — NUNCA "eyJ" (JWT)
Refresh automático sin intervención del usuario
Admin ve todo, User ve solo lo que tiene permiso
```

### Landing
```
Formulario → lead guardado en DB
Email de bienvenida en logs (desarrollo) o inbox (producción)
Lighthouse Performance > 90
Rate limit: 4to request en 1 minuto → 429
```

### Infra (MVP en producción)
```
just deploy → funciona desde la laptop
https://tudominio.com/health → {"status":"ok"}
kamal rollback → < 10 segundos
Imagen Docker < 15MB
litestream snapshots → entradas de hoy
```

---

## Reglas de oro — el contrato del proyecto

| # | Regla | Garantizada por | ADR |
|---|-------|----------------|-----|
| 1 | `crates/domain` sin deps externas | Cargo.toml domain | 0001 |
| 2 | SQL solo en `crates/database` | Cargo.toml domain/application | 0001 |
| 3 | JWT prohibido — solo PASETO v4 Local | `jsonwebtoken` fuera del workspace | 0008 |
| 4 | Soft Delete — nunca DELETE real en users | Trigger deleted_at | 0006 |
| 5 | Toda acción autenticada se audita | audit_middleware automático | 0006 |
| 6 | Tipos TypeScript por buf generate | CI verifica diff en api.ts | 0027 |
| 7 | cargo-deny + cargo-audit en CI | just audit antes de deploy | 0010 |
| 8 | Imagen distroless — ~10MB, sin shell | Containerfile | 0013 |
| 9 | Fail-fast en config al arrancar | AppConfig::load() | 0002 |
| 10 | No añadir Fase 2 hasta que el problema exista | Decisión consciente | 0011 |
| 11 | Desktop/Mobile solo con MVP web validado | ADR 0030 | 0030 |
| 12 | CLI solo después de 3 módulos a mano | ADR 0028 | 0028 |
| 13 | Límite de 200 LoC por archivo (Atomicidad) | Sintonía check arch | 0011 |

---

## Cuándo pasar de Fase — Checklist de Transición

> **Referencia:** ADR 0031 (Estrategia de Escalamiento), ADR 0011 (Estándares)
>
> No escalar prematuramente. Cada fase tiene criterios concretos de activación.
> Escalar antes del umbral es deuda técnica disfrazada de arquitectura.

### De Fase 1 (MVP) → Fase 2 (NATS + Workers)

**Criterio de activación:** Jobs pesados degradan la latencia HTTP.

```
[ ] Latencia P95 del API >50ms durante procesamiento de jobs en Apalis
    └─ Medir: hey -n 10000 -c 100 http://localhost:8080/health
    └─ Ref: ADR 0031 Nivel 4

[ ] El problema existe CON datos reales de producción
    └─ No especulación. Métricas de Sentry/OTel confirman el cuello de botella
    └─ Ref: ADR 0011 — decisión basada en datos

[ ] MVP web está validado por >30 días con usuarios reales
    └─ Ref: ADR 0030 — Fase 1 antes de Desktop/Mobile/Fase 2

[ ] Checklist previo:
    [ ] 01-ROADMAP-GENESIS.md completado ✅
    [ ] 03-ROADMAP-BACKEND.md completado ✅
    [ ] 04-ROADMAP-FRONTEND.md completado ✅
    [ ] 05-ROADMAP-AUTH-FULLSTACK.md completado ✅
    [ ] 06-ROADMAP-LANDING.md completado ✅
    [ ] 07-ROADMAP-INFRA.md completado ✅
    └─ Ref: ADR 0031 — estabilidad antes de escalar

[ ] Recursos disponibles:
    [ ] Tiempo: 1-2 semanas dedicadas
    [ ] NATS agregado a infraestructura (compose.nats.yml)
    [ ] Presupuesto VPS: puede añadir $5-10/mes para worker adicional
```

**Si el criterio NO se cumple:** Seguir en Fase 1. Optimizar Apalis config o añadir índices DB.

---

### De Fase 2 → Fase 3 (KMP + Mobile Nativo)

**Criterio de activación:** Tauri Mobile no alcanza rendimiento requerido.

```
[ ] Listas de >10.000 elementos con scroll lento (<60fps)
    └─ Medir: Chrome DevTools → Performance → Frame rate
    └─ Ref: ADR 0030 Fase 3

[ ] Animaciones <60Hz constantes en dispositivos medianos (no flagship)
    └─ Medir: GPU profiling en Android Studio / Xcode
    └─ Ref: ADR 0031 Nivel 5

[ ] Procesamiento offline masivo que requiere UI nativa
    └─ Ejemplo: procesar 1000+ registros en SQLite local con UI de progreso
    └─ Ref: ADR 0030 — criterio concreto medido

[ ] Fase 2 estable por >60 días:
    [ ] NATS procesando >10.000 mensajes/hora sin pérdida
    [ ] Workers escalan horizontalmente sin problemas
    [ ] Nunca se ha perdido un mensaje (monitoreo de dead letter queue)
    └─ Ref: ADR 0031 — madurez antes de siguiente salto

[ ] Checklist previo:
    [ ] 08-ROADMAP-TAURI-DESKTOP.md completado ✅
    [ ] 09-ROADMAP-MOBILE.md Fases 1-2 (Tauri Mobile) completado ✅
    [ ] 50-ROADMAP-FASE2.md completado ✅
```

**Si el criterio NO se cumple:** Optimizar Svelte/Tauri. KMP es costoso (requiere Kotlin, Android Studio, Xcode).

---

### Árbol de Decisión

```
¿El sistema va lento?
    ↓
¿Están activas las optimizaciones de release? (ADR 0013)
    No → activar [profile.release] opt-level="z", lto=true
    Sí ↓
¿El VPS tiene <4GB RAM?
    Sí → subir de plan (Nivel 1) — más barato que reescribir
    No ↓
¿SQLite supera 100 writes/s sostenidos?
    Sí → Turso o PostgreSQL (Nivel 2)
    No ↓
¿Los jobs pesados degradan la latencia HTTP?
    Sí → NATS + Workers (Fase 2 / Nivel 4)
    No ↓
¿La app móvil necesita 120Hz nativos?
    Sí → KMP + UniFFI (Fase 3 / Nivel 5)
    No → el sistema escala bien para el caso de uso actual
```

---

## Documentos de referencia

| Documento | Propósito |
|-----------|-----------|
| `00-ROADMAP-TEMPLATE.md` | Template para crear nuevos roadmaps de módulos |
| `ROADMAP-GENESIS.md` | Arranque del proyecto — estructura + tooling |
| `ROADMAP-BACKEND.md` | Backend completo con checklist integrado |
| `ROADMAP-FRONTEND.md` | Frontend completo con checklist integrado |
| `ROADMAP-AUTH-FULLSTACK.md` | Login/Registro back+front coordinados |
| `ROADMAP-LANDING.md` | Landing page + leads back+front |
| `ROADMAP-INFRA.md` | Deploy, Caddy, Kamal, Litestream |
| `ROADMAP-TAURI-DESKTOP.md` | Desktop Tauri 2.0 |
| `ROADMAP-MOBILE.md` | Mobile Tauri + KMP |
| `TODO.md` | Estado real de progreso — actualizar en cada sesión |
| `docs/adr/` | 31 ADRs activos + 8 ADRs futura/ |
| `STACK.md` | Stack completo con versiones de crates |
| `ARCHITECTURE.md` | Flujos y capas de la arquitectura |
| `STRUCTURE.md` | Árbol completo del monorepo |
| `SINTONIA-CLI.md` | Referencia del CLI (Fase 2) |
| `DASHBOARD-DISENO.md` | Guía de diseño del dashboard |

---

## Roadmaps Post-MVP (no son VPS $5)

| # | Documento | Propósito | VPS $5? |
|---|-----------|-----------|---------|
| 50 | `50-ROADMAP-FASE2.md` | NATS + Workers desacoplados | 🟡 Sí ($10-20) |
| 60 | `60-ROADMAP-FASE3.md` | KMP + Mobile Nativo 120Hz | 🔴 No (requiere equipo especializado) |
| 70 | `70-ROADMAP-FUTURA.md` | Escalamiento enterprise post-Fase 3 | 🔴 No ($40-500/mes) |
| 80 | `80-ROADMAP-ADMIN.md` | Admin Dashboard — gestión users, analytics | 🟢 Opcional (sigue siendo $5) |
