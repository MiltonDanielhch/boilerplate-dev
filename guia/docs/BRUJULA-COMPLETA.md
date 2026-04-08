# Brújula del Proyecto — boilerplate

> **El mapa de navegación completo.** Documentos, ADRs y guías organizados por el momento
> en que los necesitas. Usa este archivo cuando no sepas dónde está algo o qué leer primero.
>
> **31 ADRs activos · 2 futuros · última actualización: 2026**

---

## Inventario general

| Tipo | Cantidad |
|------|---------|
| ADRs activos | 31 |
| ADRs futuros | 2 |
| Roadmaps | 9 |
| Guías y referencias | 8 |

---

## 1. Documentos del proyecto — dónde vive cada cosa

| # | Documento | Ubicación | Cuándo leerlo |
|---|-----------|-----------|---------------|
| 1 | `ROADMAP-MASTER.md` | `docs/` | **Primero siempre** — orden de ejecución y paralelismo |
| 2 | `TODO.md` | raíz | Al inicio de cada sesión — estado real del progreso |
| 3 | `PROMPT_MAESTRO.md` | `docs/` | Copiar completo al iniciar una nueva sesión con IA |
| 4 | `BRUJULA-COMPLETA.md` | `docs/adr/` | Cuando no sabes dónde está algo (este archivo) |
| 5 | `INICIO.md` | `docs/` | Al crear el primer módulo — guía paso a paso con código real |
| 6 | `STACK.md` | `docs/` | Consultar versiones exactas de crates y dependencias |
| 7 | `ARCHITECTURE.md` | `docs/` | Entender flujos completos y capas del sistema |
| 8 | `STRUCTURE.md` | `docs/` | Ver el árbol de archivos y la jerarquía de crates |
| 9 | `DASHBOARD-DISENO.md` | `docs/` | Diseño del dashboard — componentes y RBAC en UI |
| 10 | `SINTONIA-CLI.md` | `docs/` | Referencia técnica del CLI (Fase 2) |

---

## 2. Roadmaps — qué construir y cuándo

| # | Roadmap | Cuándo usarlo |
|---|---------|---------------|
| 1 | `ROADMAP-GENESIS.md` | Día 1 — workspace + tooling + estructura |
| 2 | `ROADMAP-BACKEND.md` | Dominio + DB + Auth + API (con checklists integrados) |
| 3 | `ROADMAP-FRONTEND.md` | Astro + Svelte 5 + Dashboard (paralelo con Backend II) |
| 4 | `ROADMAP-AUTH-FULLSTACK.md` | Login/Registro back+front coordinados |
| 5 | `ROADMAP-LANDING.md` | Landing page + captura de leads back+front |
| 6 | `ROADMAP-INFRA.md` | Deploy + Caddy + Kamal + Litestream |
| 7 | `ROADMAP-TAURI-DESKTOP.md` | Desktop Tauri 2.0 (solo con MVP web validado) |
| 8 | `ROADMAP-MOBILE.md` | Mobile Tauri + KMP (solo con Desktop validado) |
| 9 | `ROADMAP-MASTER.md` | Mapa de todos los mapas + orden de ejecución |

---

## 3. ADRs — organizados por fase de desarrollo

> Lee el ADR del área en la que estás trabajando.
> El número nuevo (0001-0031) es el orden lógico de lectura.

---

### Fase 0 — Antes de escribir código
*Decisiones que afectan todo lo demás. Leer primero.*

| # | ADR | Qué decide | Una línea |
|---|-----|-----------|-----------|
| 0001 | Arquitectura Hexagonal | Cómo organizar TODO el código | El dominio no conoce HTTP ni SQL — nunca |
| 0002 | Configuración Tipeada | Variables de entorno + fail-fast | Si falta una variable, el proceso no arranca |
| 0011 | Estándares de Desarrollo | Ciclo Lab→Puente→Producción | Funciones ≤30 líneas, archivos ≤200, código autodocumentado |
| 0012 | Herramientas: just + lefthook | Tooling del proyecto | `just setup` en <5 minutos, hooks que no se saltan |
| 0013 | Build Externo Distroless | Dónde se compila el binario | Prohibido compilar en el VPS — imagen ~10MB |
| **VPS** | **Protocolo 5 Pilares** | **Cómo preparar el VPS** | **Swap + SSH + Caddy + UFW + Systemd — VPS de $5 a prueba de balas** |

---

### Fase 1 — El núcleo vivo
*Servidor HTTP respondiendo, DB conectada, 6 migraciones pasando.*

| # | ADR | Qué decide | Una línea |
|---|-----|-----------|-----------|
| 0003 | Stack Backend Rust+Axum | Framework HTTP + Tokio | Middleware composable con Tower — graceful shutdown |
| 0004 | Persistencia SQLite+Litestream | Base de datos + backup | SQLite WAL + queries compile-time + RPO 1 segundo |
| 0005 | Migraciones SQLx | Evolución del esquema | Migraciones automáticas al arrancar, seeds idempotentes |
| 0006 | RBAC + Sessions + Audit | Las 6 migraciones base | Esquema completo de auth desde el día uno |
| 0007 | Jerarquía de Errores | DomainError → AppError → HTTP | JSON consistente `{ "error": "code", "message": "..." }` |
| 0010 | Testing 4 Capas | Estrategia de tests | Domain (sin deps) → Mocks → SQLite → E2E |
| 0016 | Observabilidad | Logs + Sentry + OTLP | ~20MB RAM total — Sentry para errores, Axiom para trazas |

---

### Fase 2 — La primera célula (Auth + RBAC)
*El usuario puede registrarse, iniciar sesión y las rutas están protegidas.*

| # | ADR | Qué decide | Una línea |
|---|-----|-----------|-----------|
| 0008 | Seguridad PASETO+argon2id | Auth sin JWT | PASETO v4 — payload cifrado, sin agilidad de algoritmos |
| 0009 | Rate Limiting | Protección de endpoints | 1 req/s en auth (anti fuerza bruta), 10 req/s en API |
| 0021 | OpenAPI Utoipa+Scalar | Documentación API | `/docs` generado desde código Rust — IA-ready |

---

### Fase 3 — La conexión (Frontend + Caché + Email)
*El usuario ve una UI, los datos están en caché, los emails funcionan.*

| # | ADR | Qué decide | Una línea |
|---|-----|-----------|-----------|
| 0017 | Caché Moka Decorator | Cache in-process | Invisible para el dominio — TTL 5min, 100MB máximo |
| 0018 | Jobs Apalis+SQLite | Procesamiento asíncrono | Jobs sin Redis — durables, 3 reintentos, SQLite |
| 0019 | Email Resend+LogMailer | Envío de emails | Puerto abstracto — LogMailer en dev, Resend en prod |
| 0022 | Frontend Astro+Svelte 5 | Stack del frontend | HTML primero, JS solo donde sea necesario |
| 0023 | i18n Paraglide JS | Internacionalización | Traducciones compiladas — BOB, DD/MM/YYYY, La Paz |
| 0029 | Landing Page + Leads | Captura de leads | Entidad Lead separada de User — formulario con honeypot |

---

### Fase 4 — Almacenamiento y resiliencia
*Los archivos están en S3, el monitoreo alerta antes de que algo explote.*

| # | ADR | Qué decide | Una línea |
|---|-----|-----------|-----------|
| 0020 | Storage Tigris S3 | Archivos binarios | Sin costos de egress — presigned URLs — R2 como fallback |
| 0015 | Monitoreo Healthchecks.io | Alertas de tareas críticas | Si el ping no llega → algo explotó → alerta inmediata |
| 0014 | Deploy Podman+Caddy+Kamal | Infraestructura de producción | Zero-downtime, rollback en 5 segundos, distroless |

---

### Fase 5 — Local-First y eventos
*La app funciona offline, los módulos se comunican de forma asíncrona.*

| # | ADR | Qué decide | Una línea |
|---|-----|-----------|-----------|
| 0024 | Local-First SQLite Wasm | App offline | SQLite en el navegador — funciona sin internet |
| 0025 | Eventos NATS JetStream | Bus de eventos (Fase 2) | Persistente en ~25MB RAM — `msg.ack()` obligatorio |
| 0026 | NATS Publisher | Detalles del publicador | Complemento del ADR 0025 — leer juntos |
| 0027 | ConnectRPC + buf generate | Contratos multiplataforma | Un `.proto` genera tipos para Rust, TypeScript, Kotlin, Swift |

---

### Fase 6 — Automatización y CLI
*El CLI genera módulos completos con RBAC incluido.*

| # | ADR | Qué decide | Una línea |
|---|-----|-----------|-----------|
| 0028 | Sintonía CLI | Generador de módulos | `sintonia g module` desde el módulo 4 — conoce RBAC |

---

### Fase 7 — Multiplataforma
*Desktop y Mobile comparten el mismo dominio Rust.*

| # | ADR | Qué decide | Una línea |
|---|-----|-----------|-----------|
| 0030 | Multiplataforma Tridente | Desktop + Mobile | Tauri <15MB — KMP+UniFFI solo si se necesita 120Hz |

---

### Fase 8 — Escalamiento (activar con criterio concreto)

| # | ADR | Qué decide | Una línea |
|---|-----|-----------|-----------|
| 0031 | Escalamiento 5 Niveles | Cómo escalar sin reescribir | Vertical → Turso → Postgres → Worker → KMP |
| F-001 | SurrealDB + RocksDB | Alternativa de DB futura | Cuando SQLite sea insuficiente para grafos |
| F-002 | PostgreSQL Escala | Escala horizontal futura | >100 writes/s sostenidos — solo el adaptador cambia |

---

## 4. Mapa de dependencias entre ADRs

```
ADR 0001 (Arquitectura Hexagonal)
  └─ base de TODO lo demás — leer primero

ADR 0002 (Config) ──────────────── arranque del proceso
ADR 0011 (Estándares) ──────────── disciplina de código
ADR 0012 (just/lefthook) ───────── workflow diario
ADR 0013 (Build externo) ───────── compile-time en local/CI
ADR 0005 (Migraciones) ─────────── evolución del esquema
  └─ depende de: ADR 0004 (SQLite)

ADR 0003 (Axum) ────────────────── servidor HTTP
  └─ middleware de: ADR 0008 (Auth), ADR 0009 (Rate limit)
  └─ spans de: ADR 0016 (Tracing)

ADR 0008 (Auth) ────────────────── autenticación
  └─ tokens en: ADR 0004 (SQLite)
  └─ cleanup en: ADR 0018 (Apalis)

ADR 0007 (Errores) ─────────────── formato de respuesta HTTP
ADR 0006 (RBAC) ────────────────── 6 migraciones base
  └─ depende de: ADR 0004, ADR 0008
  └─ usado por: ADR 0028 (CLI genera permisos)

ADR 0017 (Moka) ────────────────── caché L1
  └─ envuelve: repositorios de ADR 0004

ADR 0018 (Apalis) ──────────────── jobs asíncronos
  └─ backend en: ADR 0004 (SQLite)
  └─ email via: ADR 0019 (Resend)

ADR 0022 (Astro+Svelte) ────────── frontend
  └─ tipos de: ADR 0027 (ConnectRPC/buf)
  └─ tokens de: ADR 0008 (PASETO)
  └─ i18n de: ADR 0023 (Paraglide)
  └─ offline: ADR 0024 (Local-First)

ADR 0014 (Deploy) ──────────────── producción
  └─ binario de: ADR 0003 + ADR 0013 (distroless)
  └─ backup de: ADR 0004 (Litestream)
  └─ secrets de: ADR 0002 (Config)
  └─ monitoreo: ADR 0015 (Healthchecks)
  └─ archivos: ADR 0020 (Tigris)

ADR 0030 (Multiplataforma) ──────── desktop + móvil
  └─ reutiliza: crates/domain (sin cambios)
  └─ Fase 1: Tauri 2.0
  └─ Fase 3: KMP + UniFFI

ADR 0028 (CLI) ─────────────────── automatización
  └─ conoce: ADR 0006 (RBAC — genera permisos)
  └─ verifica: ADR 0001 (arquitectura)
```

---

## 5. Guía de inicio rápido — de cero a producción

```bash
# ── Día 1 — Génesis ──────────────────────────────────────────────────────────
# Lee: ROADMAP-GENESIS.md
git init boilerplate && cd boilerplate
just setup                    # ADR 0012 — instala todo en <5 minutos

# ── Día 2 — Fundación (crítico) ──────────────────────────────────────────────
# Lee: ROADMAP-BACKEND.md (Bloque I)
just migrate                  # ADR 0006 — 6 migraciones, admin@admin.com listo
# Luego: dominio → repositorios → casos de uso

# ── Días 3-4 — API + Auth ────────────────────────────────────────────────────
# Lee: ROADMAP-BACKEND.md (Bloques II-III) + ROADMAP-AUTH-FULLSTACK.md
just dev-api                  # ADR 0003 — servidor HTTP corriendo
curl http://localhost:8080/health  # {"status":"ok","database":"connected"}

# ── Día 5 — Frontend (paralelo con Backend) ───────────────────────────────────
# Lee: ROADMAP-FRONTEND.md
cd apps/web && pnpm dev       # ADR 0022 — Astro SSR + Svelte 5

# ── Días 6-7 — Landing + Jobs ────────────────────────────────────────────────
# Lee: ROADMAP-LANDING.md + ROADMAP-BACKEND.md (Bloque V)

# ── Día 8 — Deploy ───────────────────────────────────────────────────────────
# Lee: ROADMAP-INFRA.md + ADR VPS (Protocolo 5 Pilares)
just deploy                   # ADR 0014 — zero-downtime en VPS de $5
```

---

## 6. Preguntas frecuentes → dónde está la respuesta

| Pregunta | Documento | ADR |
|----------|-----------|-----|
| ¿Cómo agrego una nueva variable de entorno? | `STACK.md` | 0002 |
| ¿Por qué no puedo usar JWT? | `INICIO.md` Paso 14 | 0008 |
| ¿Cómo creo un módulo nuevo completo? | `INICIO.md` Partes 2-3 | 0001 |
| ¿Dónde va el SQL? | `ARCHITECTURE.md` | 0001, 0004 |
| ¿Cómo funciona el RBAC exactamente? | `INICIO.md` Paso 12 | 0006 |
| ¿Por qué el token empieza con `v4.local.`? | `ROADMAP-AUTH-FULLSTACK.md` A.2 | 0008 |
| ¿Cómo agrego un nuevo permiso? | `ROADMAP-BACKEND.md` I.2 | 0006 |
| ¿Dónde van los tests de integración? | `INICIO.md` Paso 20 | 0010 |
| ¿Cómo configuro el deploy en el VPS? | `ROADMAP-INFRA.md` | 0014, VPS |
| ¿Cuándo activo NATS? | `ADR-0025` | 0025 |
| ¿Cuándo activo el CLI? | `SINTONIA-CLI.md` | 0028 |
| ¿Cuándo activo Desktop/Mobile? | `ROADMAP-MASTER.md` | 0030 |
| ¿Cuándo migrar de SQLite? | `ADR-0031` | 0031 |
| ¿Cómo preparar el VPS de $5? | `ADR-VPS-5-PILARES.md` | VPS |

---

## 7. Checklist de arranque del proyecto

```bash
# 1. Verificar que nada tiene JWT
grep -r "jsonwebtoken" . --include="*.toml"  → cero resultados

# 2. Verificar que el dominio no tiene sqlx
grep "sqlx" crates/domain/Cargo.toml         → cero resultados

# 3. Las 6 migraciones ejecutadas
just migrate                                  → 6 "Applied"

# 4. Tests pasando
cargo nextest run                             → verde

# 5. Calidad garantizada
cargo deny check && cargo audit               → sin violations

# 6. Servidor respondiendo
curl http://localhost:8080/health             → {"status":"ok"}

# 7. Token PASETO (no JWT)
POST /auth/login → access_token empieza con "v4.local."
# NUNCA con "eyJ" — si ves "eyJ", algo está mal
```

---

## 8. Reglas de oro — el contrato del proyecto

| # | Regla | ADR |
|---|-------|-----|
| 1 | `crates/domain` sin dependencias externas — Cargo.toml lo garantiza | 0001 |
| 2 | SQL solo en `crates/database` | 0001 |
| 3 | JWT prohibido — solo PASETO v4 Local (`pasetors`) | 0008 |
| 4 | Soft Delete — `UPDATE deleted_at`, nunca `DELETE` real en users | 0006 |
| 5 | Toda acción autenticada se audita automáticamente | 0006 |
| 6 | Tipos TypeScript por `buf generate` — nunca escritos a mano | 0027 |
| 7 | `cargo-deny` + `cargo-audit` en CI siempre | 0010 |
| 8 | Imagen distroless — ~10MB, sin shell | 0013 |
| 9 | Fail-fast en config — si falta variable, el proceso no arranca | 0002 |
| 10 | No añadir Fase 2 hasta que el problema concreto exista | 0011 |
| 11 | Desktop/Mobile solo con MVP web en producción y validado | 0030 |
| 12 | CLI solo después de 3 módulos hechos a mano | 0028 |
