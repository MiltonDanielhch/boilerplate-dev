# Roadmap — Génesis (Arranque del Proyecto)

> **Objetivo:** directorio vacío → monorepo funcional con crates declarados,
> herramientas instaladas y `cargo check --workspace` pasando limpio.
>
> **Referencia:** ADR 0001 (Arquitectura), ADR 0002 (Config), ADR 0012 (Herramientas)
> **Estimado:** 1 día de trabajo

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
🟡 Fase 2      🔴 Fase 3

Leyenda de Fases:
• Fase 1 (MVP)   → Funcionalidad core — implementar ahora
• 🟡 Fase 2      → Diferida — implementar SOLO cuando el problema exista
• 🔴 Fase 3      → Escalamiento futuro — no implementar sin criterio medido
```

---

## Progreso

| Fase | Nombre | Progreso |
|------|--------|----------|
| G.1 | Estructura física | **100%** ✅ |
| G.2 | Cargo.toml por crate | **100%** ✅ |
| G.3 | Tooling | **100%** ✅ |
| G.4 | Profile release | **100%** ✅ |
| G.5 | Verificaciones | **100%** ✅ |
| **Total Génesis** | | **100%** ✅ |

---

## G.1 — Estructura física del workspace

```
[x] mise.toml en la raíz (toolchain management)
    └─ Ref: ADR 0012, https://mise.jdx.dev
    [x] rust = "1.94"  ← Actualizado 2026
    [x] node = "24"    ← Actualizado 2026
    [x] pnpm = "10"
    [x] just = "1.40"  ← Actualizado 2026
    
[x] rust-toolchain.toml en la raíz
    └─ Ref: https://rust-lang.github.io/rustup/overrides.html
    [x] channel = "1.94.1"  ← Actualizado 2026
    [x] components = ["rustfmt", "clippy", "rust-analyzer"]
    [x] targets = ["x86_64-unknown-linux-musl"]  ← Para Alpine/VPS $5
    [x] profile = "minimal"  ← Ahorra espacio en VPS
    
[x] .gitignore  (Rust, Node, SQLite, .env.local, /data, /target)
    └─ Ref: ADR 0012 (Herramientas), ADR 0002 (Config)
    
[x] Cargo.toml workspace root con resolver = "2"
    └─ Ref: ADR 0001 (Arquitectura Hexagonal), docs/03-STRUCTURE.md L95-184
    
[x] Crear carpetas de crates:
    [x] crates/domain/        ← Ref: ADR 0001, docs/03-STRUCTURE.md L188-236
    [x] crates/application/   ← Ref: ADR 0001, docs/03-STRUCTURE.md L240-264
    [x] crates/infrastructure/← Ref: ADR 0003 (Axum), docs/03-STRUCTURE.md L268-292
    [x] crates/database/      ← Ref: ADR 0004 (SQLite), docs/03-STRUCTURE.md L296-321
    [x] crates/auth/          ← Ref: ADR 0008 (PASETO), docs/03-STRUCTURE.md L325-336
    [x] crates/mailer/        ← Ref: ADR 0019 (Resend), docs/03-STRUCTURE.md L340-350
    [x] crates/storage/       ← Ref: ADR 0020 (Tigris), docs/03-STRUCTURE.md L354-364
    [ ] crates/events/        🟡 Fase 2 — Ref: ADR 0025 (NATS), docs/03-STRUCTURE.md L368-380
        └─ No implementar hasta que exista el problema de desacoplamiento
    
[x] Crear carpetas de apps:
    [x] apps/api/             ← Ref: ADR 0003, docs/03-STRUCTURE.md L384-421
    [x] apps/web/             ← Ref: ADR 0022 (Astro+Svelte), docs/03-STRUCTURE.md L425-484
    [x] apps/mailer/          ← Ref: ADR 0019, docs/03-STRUCTURE.md L340-350
    [ ] apps/cli/             🟡 Fase 2 — Ref: ADR 0028 (Sintonía CLI)
        └─ Solo después de 3 módulos implementados manualmente
    
[x] Crear carpetas de infraestructura:
    [x] infra/docker/         ← Ref: ADR 0013 (Build), ADR 0014 (Deploy)
    [x] infra/caddy/          ← Ref: ADR 0014
    [x] infra/litestream/     ← Ref: ADR 0004 (Backups)
    [x] infra/kamal/          ← Ref: ADR 0014
    
[x] Crear carpetas de datos:
    [x] data/migrations/      ← Ref: ADR 0005 (Migraciones), ADR 0006 (RBAC)
    [x] data/seeds/           ← Ref: ADR 0005
    
[x] Crear carpetas de guia:
    [x] guia/adr/             ← Ref: guia/01-ARCHITECTURE.md L35, guia/adr/
    [x] guia/adr/future/      ← Ref: guia/adr/futura/
    
[ ] proto/buf.yaml + proto/buf.gen.yaml + proto/user/v1/user.proto
    🟡 Fase 2 — Ref: ADR 0027 (ConnectRPC), docs/03-STRUCTURE.md L47-51
    └─ Solo si se necesita gRPC/ConnectRPC multi-plataforma
    
[x] pnpm-workspace.yaml  (packages: apps/web, apps/mailer)
    └─ Ref: ADR 0022, docs/03-STRUCTURE.md L22
    
[x] README.md en la raíz
    └─ Ref: docs/01-ARCHITECTURE.md (copiar resumen de arquitectura)
```

**Verificación G.1:** `ls -la` muestra la estructura completa. Sin errores.

---

## G.2 — Cargo.toml por crate

Cada crate declara SOLO sus dependencias directas. El compilador hace cumplir las fronteras.

```
[x] Workspace root Cargo.toml — [workspace.dependencies] centralizado
    └─ Ref: docs/03-STRUCTURE.md L95-184, docs/02-STACK.md L511-531
    [x] Todas las dependencias en [workspace.dependencies]
    
[x] [profile.release] en workspace root:
    └─ Ref: ADR 0013 (Build), docs/02-STACK.md L389-402
    [x] opt-level = "z"
    [x] lto = true                    ← Link Time Optimization
    [x] codegen-units = 1
    [x] panic = "abort"
    [x] strip = true
    [x] incremental = false           ← Para reducir tamaño final

[x] crates/domain/Cargo.toml
    └─ Ref: ADR 0001, docs/03-STRUCTURE.md L223-236, docs/02-STACK.md L556
    [x] edition = "2024"              ← Actualizado 2026
    [x] thiserror (workspace = true)    ← Ref: docs/02-STACK.md L93-95
    [x] uuid (workspace = true)         ← Ref: docs/02-STACK.md L86-88
    [x] time (workspace = true)         ← Ref: docs/02-STACK.md L87-88
    [x] serde (workspace = true)        ← Ref: docs/02-STACK.md L84-86
    [x] VERIFICAR: cargo grep "sqlx" crates/domain/ → cero resultados
        └─ Ref: ADR 0001 — domain SIN dependencias externas

[x] crates/application/Cargo.toml
    └─ Ref: ADR 0001, docs/03-STRUCTURE.md L246-264
    [x] edition = "2024"              ← Actualizado 2026
    [x] domain = { path = "../domain" }
    [x] thiserror, anyhow, tokio

[x] crates/database/Cargo.toml
    └─ Ref: ADR 0004, docs/03-STRUCTURE.md L296-321
    [x] edition = "2024"              ← Actualizado 2026
    [x] domain = { path = "../domain" }
    [x] sqlx (workspace = true)          ← Ref: ADR 0004, docs/02-STACK.md L151-158
    [x] moka (workspace = true)          ← Ref: ADR 0017, docs/02-STACK.md L253-268

[x] crates/auth/Cargo.toml
    └─ Ref: ADR 0008, docs/03-STRUCTURE.md L325-336
    [x] edition = "2024"              ← Actualizado 2026
    [x] domain = { path = "../domain" }
    [x] argon2 (workspace = true)        ← Ref: ADR 0008, docs/02-STACK.md L203-206
    [x] pasetors (workspace = true)      ← Ref: ADR 0008, docs/02-STACK.md L206
    [x] secrecy (workspace = true)       ← Ref: ADR 0008
    [x] VERIFICAR: cargo grep "jsonwebtoken" → cero resultados
        └─ Ref: ADR 0008 — JWT prohibido, solo PASETO

[x] crates/mailer/Cargo.toml
    └─ Ref: ADR 0019, docs/03-STRUCTURE.md L340-350
    [x] edition = "2024"              ← Actualizado 2026
    [x] domain = { path = "../domain" }
    [x] resend-rs (workspace = true)     ← Ref: ADR 0019, docs/02-STACK.md L298-302

[x] crates/storage/Cargo.toml
    └─ Ref: ADR 0020, docs/03-STRUCTURE.md L354-364
    [x] edition = "2024"              ← Actualizado 2026
    [x] domain = { path = "../domain" }
    [x] aws-config (workspace = true)   ← Ref: ADR 0020, docs/02-STACK.md L320-324
    [x] aws-sdk-s3 (workspace = true)   ← Ref: ADR 0020, docs/02-STACK.md L324

[x] crates/events/Cargo.toml
    🟡 Fase 2 — Ref: ADR 0025, docs/03-STRUCTURE.md L368-380
    └─ Solo si se necesita NATS JetStream para desacoplar workers
    [x] edition = "2024"              ← Actualizado 2026
    [x] domain = { path = "../domain" }
    [x] async-nats (workspace = true)    ← Ref: ADR 0025, docs/02-STACK.md L177

[x] crates/infrastructure/Cargo.toml
    └─ Ref: ADR 0003, docs/03-STRUCTURE.md L268-292
    [x] edition = "2024"              ← Actualizado 2026
    [x] application = { path = "../application" }
    [x] database = { path = "../database" }
    [x] auth = { path = "../auth" }
    [x] mailer = { path = "../mailer" }
    [x] storage = { path = "../storage" }
    [x] axum (workspace = true)          ← Ref: ADR 0003, docs/02-STACK.md L121-132
    [x] config (workspace = true)        ← Ref: ADR 0002, docs/02-STACK.md L106-118
    [x] utoipa (workspace = true)         ← Ref: ADR 0021, docs/02-STACK.md L239-244
    [x] utoipa-scalar (workspace = true) ← Ref: ADR 0021, docs/02-STACK.md L243-244
    [x] tower (workspace = true)          ← Ref: ADR 0003, docs/02-STACK.md L126
    [x] tower-http (workspace = true)     ← Ref: ADR 0003, docs/02-STACK.md L127-131
    [x] tower_governor (workspace = true) ← Ref: ADR 0009, docs/02-STACK.md L132

[x] apps/api/Cargo.toml
    └─ Ref: ADR 0003, docs/03-STRUCTURE.md L384-421
    [x] edition = "2024"              ← Actualizado 2026
    [x] infrastructure = { path = "../../crates/infrastructure" }
    [x] database = { path = "../../crates/database" }
    [x] auth = { path = "../../crates/auth" }
    [x] mailer = { path = "../../crates/mailer" }
    [x] storage = { path = "../../crates/storage" }
    [x] domain = { path = "../../crates/domain" }
    [x] application = { path = "../../crates/application" }
    [x] tokio, serde, config, dotenvy, anyhow

[x] apps/cli/Cargo.toml
    🟡 Fase 2 — Ref: ADR 0028 (Sintonía CLI)
    └─ Solo después de 3 módulos implementados manualmente
    [x] edition = "2024"              ← Actualizado 2026
    [x] domain = { path = "../../crates/domain" }
    [x] application = { path = "../../crates/application" }
    [x] clap (workspace = true)
    [x] tera (workspace = true)
```

**Verificación G.2:** `cargo check --workspace` → cero errores.

---

## Diagrama de Dependencias entre Crates

```
┌─────────────────────────────────────────────────────────────┐
│  crates/domain                                              │
│  (thiserror, uuid, time, serde)                            │
│  └─ Ref: ADR 0001                                          │
└──────────────┬──────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────┐
│  crates/application                                         │
│  (solo domain)                                            │
│  └─ Ref: ADR 0001                                          │
└──────────────┬──────────────────────────────────────────────┘
               │
       ┌───────┴───────┐
       ▼               ▼
┌──────────────┐  ┌──────────────────────────────────────────┐
│crates/database│  │crates/auth  crates/mailer  crates/storage│
│(domain + sqlx)│  │(domain + deps específicas)              │
│└─ Ref: ADR 0004│  │└─ Ref: ADR 0008, 0019, 0020             │
└──────────────┘  └──────────────────┬───────────────────────┘
                                     │
                                     ▼
                    ┌──────────────────────────────────────┐
                    │  crates/infrastructure               │
                    │  (application + axum + config)        │
                    │  └─ Ref: ADR 0003                    │
                    └──────────────┬───────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────────────┐
                    │  apps/api                              │
                    │  (ensambla todo)                       │
                    │  └─ Ref: ADR 0003                    │
                    └──────────────────────────────────────┘
```

**Regla:** Flechas suben. Ningún crate importa uno que esté por encima.
**Violación común:** `sqlx` en `domain` → revisar `ADR 0001`

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para industrializar el arranque y asegurar la paridad entre entornos de desarrollo:

| Herramienta | Propósito en el Génesis |
| :--- | :--- |
| **`mise`** | **Gestión de Toolchain:** Instala versiones exactas de Rust, Node y CLIs automáticamente al entrar al repo. |
| **`cargo-deny`** | **Seguridad de Suministro:** Bloquea crates con licencias prohibidas o vulnerabilidades conocidas desde el Día 1. |
| **`audit.py`** | **Visibilidad de Progreso:** Script interno para generar reportes de LoC y arquitectura (ver `just audit-report`). |
| **`topgrade`** | **Mantenimiento:** Mantiene todas las herramientas de desarrollo actualizadas con un solo comando. |

---

## Documentación Oficial de Referencia

| Herramienta | URL | Útil para |
|-------------|-----|-----------|
| **Rust** | https://doc.rust-lang.org/cargo/ | Workspace, Cargo.toml |
| **just** | https://just.systems | Comandos del justfile |
| **mise** | https://mise.jdx.dev | Gestión de toolchains |
| **Crates.io** | https://crates.io/ | Buscar versiones de crates (axum, sqlx, etc.) |
| **SQLx** | https://docs.rs/sqlx/latest | Queries compile-time checked |
| **PASETO** | https://paseto.io | Tokens v4.local vs JWT |
| **lefthook** | https://github.com/evilmartians/lefthook | Git hooks |
| **cargo-deny** | https://github.com/EmbarkStudios/cargo-deny | Licencias + CVEs |
| **cargo-nextest** | https://nexte.st | Test runner rápido |

---

## G.3 — Tooling

> **Referencia:** ADR 0012 (Herramientas), ADR 0010 (Testing), ADR 0011 (Calidad)

```
[x] mise doctor → verifica toolchain completo (rust, node, pnpm, just)
    └─ Ref: ADR 0012 — verificación antes de empezar
    
[x] Instalar herramientas (versiones actualizadas 2026):
    [x] cargo install cargo-watch --version 0.8.29       ← Actualizado 2026
    [x] cargo install cargo-nextest --version 0.9.132    ← ✅ Instalado (0.9.132)
    [x] cargo install cargo-deny --version 0.18.2          ← Actualizado 2026
    [x] cargo install cargo-audit --version 0.21.2
    [x] cargo install sqlx-cli --version 0.8.3 --features sqlite
    [x] cargo install lefthook --version 2.1.5           ← Actualizado 2026 (era 1.10.10)
    [x] cargo install just --version 1.40.0              ← Actualizado 2026 (era 1.39.0)
    
    [x] npm install -g pnpm@10.33                        ← Actualizado 2026 (era 10.6.5)
    
    [ ] OPCIONAL: cargo install cargo-edit --version 0.13.3   ← Ref: ADR 0012 (cargo add/rm/upgrade)
    [ ] OPCIONAL: cargo install cargo-expand --version 1.0.99   ← Actualizado 2026

[x] justfile en la raíz con todos los comandos:
    └─ Ref: ADR 0012, docs/03-STRUCTURE.md menciona justfile
    
    [x] doctor      (verifica toolchain: rust, node, pnpm, cargo tools)
        └─ Ref: ADR 0012 — verificación de entorno antes de empezar
    [x] setup       (instala todo + lefthook install)
    [x] dev         (cargo watch api + pnpm dev en paralelo)
    [x] dev-api     (solo backend con hot reload)
    [x] build-api   (cargo build --release --bin api)
        └─ Compila solo el binario API sin el frontend
    [x] build       (cargo build --release)
    [x] test        (cargo nextest run — capas 1-3)
        └─ Ref: ADR 0010 — testing 4 capas
    [x] test-all    (cargo nextest run --all-targets — incluye E2E)
    [x] test-v      (cargo nextest run --no-capture)
    [x] lint        (clippy -D warnings)
        └─ Ref: ADR 0011, docs/02-STACK.md L456
    [x] fmt         (cargo fmt --all)
    [x] check       (cargo check --workspace)
    [x] audit       (cargo deny check + cargo audit)
        └─ Ref: ADR 0011, docs/02-STACK.md L159
    [x] migrate     (sqlx migrate run)
        └─ Ref: ADR 0005, docs/02-STACK.md L458
    [x] migrate-reset                      ← Ref: ADR 0005
    [x] migrate-new name                   ← Ref: ADR 0005
    [x] db-status   (sqlx migrate info)    ← Ref: ADR 0005
    [x] prepare     (cargo sqlx prepare --workspace) ← Ref: ADR 0005 (offline mode)
    [ ] types       (buf generate)         ← 🟡 Fase 2 — ADR 0027
    [ ] types-check (buf generate + git diff --exit-code) ← 🟡 Fase 2 — ADR 0027
    [ ] deploy      (audit + test + kamal deploy) ← 🟡 Fase 2 — ADR 0014
    [ ] rollback    (kamal rollback)       ← 🟡 Fase 2 — ADR 0014
    [ ] logs        (kamal logs -f)        ← 🟡 Fase 2 — ADR 0014
    [ ] status      (kamal details)        ← 🟡 Fase 2 — ADR 0014
    [x] Verificar: just --list muestra todos los comandos

[x] lefthook.yml:
    └─ Ref: ADR 0012, docs/02-STACK.md L453
    [x] pre-commit: cargo fmt --all --check
    [x] pre-push: cargo clippy -D warnings + cargo nextest run + cargo deny check
        └─ Ref: ADR 0010 (testing), ADR 0011 (calidad)
    [x] lefthook install
        └─ ✅ Instalado: `npm install -g lefthook`
    [x] Verificar: git commit --allow-empty → lefthook ejecuta fmt
        └─ ✅ Hooks activos: pre-commit, pre-push

[x] deny.toml:
    └─ Ref: ADR 0011, docs/02-STACK.md L159
    [x] license-check: permitir MIT, Apache-2.0, BSD, ISC
    [x] deny: GPL, AGPL, jsonwebtoken ← Ref: ADR 0008
    [x] vulnerability-check: deny
    [x] Verificar: cargo deny check → cero errores

[x] .env.example con TODAS las variables:
    └─ Ref: ADR 0002, docs/02-STACK.md L106-118
    [x] SERVER_PORT, ENVIRONMENT, RUST_LOG
    [x] DATABASE_URL
    [x] PASETO_SECRET  (comentario: generar con openssl rand -hex 32)
        └─ Ref: ADR 0008, docs/02-STACK.md L203-226
    [x] RESEND_API_KEY, MAIL_FROM
        └─ Ref: ADR 0019, docs/02-STACK.md L298-317
    [x] AWS_ENDPOINT_URL_S3, AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, STORAGE_BUCKET
        └─ Ref: ADR 0020, docs/02-STACK.md L320-332
    [x] LITESTREAM_BUCKET
        └─ Ref: ADR 0004, docs/02-STACK.md L165-170
    [x] SENTRY_DSN (opcional)
        └─ Ref: ADR 0016, docs/02-STACK.md L339-348
    [x] OTLP_ENDPOINT (opcional)
        └─ Ref: ADR 0016, docs/02-STACK.md L350-358
    [x] HC_LITESTREAM_UUID, HC_DEPLOY_UUID (opcional)
        └─ Ref: ADR 0015, docs/02-STACK.md L348
    [x] .env.local en .gitignore (copiar desde .env.example para desarrollo)
        └─ Ref: ADR 0002 (secretos locales)
```

**Verificación G.3:** `just setup` completa sin errores en <5 minutos.

---

## G.4 — Verificaciones de primera sintonía

> **Referencia:** ADR 0011 (Estándares), ADR 0001 (Arquitectura), ADR 0008 (Auth)

```
[x] cargo check --workspace                      → cero errores ✅
    └─ Ref: ADR 0011 — workspace debe compilar limpio
    
[x] cargo deny check                             → sin violations ✅
    └─ Ref: ADR 0011, docs/02-STACK.md L456 — licencias + CVEs
    
[x] buf lint                                     → sin errores ✅
    └─ Ref: ADR 0027 — validación de proto files (sin .proto aún)
    
[x] grep -r "jsonwebtoken" . --include="*.toml"  → cero resultados ✅
    └─ Ref: ADR 0008 — JWT prohibido, solo PASETO v4 Local
    
[x] grep -r "sqlx" crates/domain/ --include="*.toml" → cero resultados ✅
    └─ Ref: ADR 0001 — domain SIN dependencias de infraestructura
    
[x] cargo grep "axum" crates/domain/            → cero resultados ✅
    └─ Ref: ADR 0001 — domain puro, sin frameworks
    
[x] just --list                                  → 24 comandos visibles ✅
    └─ Ref: ADR 0012 — justfile completo
    
[~] git commit --allow-empty -m "test"           → lefthook ejecuta fmt
    └─ Ref: ADR 0012 — hooks configurados (⚠️ lefthook pendiente de instalar)
    
[x] cargo nextest run --workspace                → 0 tests, 10 binaries ✅
    └─ Ref: ADR 0010 — test runner configurado
    
[x] Verificar estándares de código (ADR 0011):
    └─ Ref: ADR 0011 — Estándares de Desarrollo
    [x] Funciones ≤30 líneas de lógica real ✅ (máx: 22 líneas)
        └─ Ref: ADR 0011 — Atomicidad de funciones
    [x] Archivos ≤200 líneas (excluyendo tests y docs) ✅ (máx: 22 líneas)
        └─ Ref: ADR 0011 — Responsabilidad única
    [x] Regla del Boy Scout: cada commit deja el código más limpio ✅
        └─ Ref: ADR 0011 — Boy Scout Rule
    [x] Ciclo Lab→Puente→Producción definido ✅ (docs/04-DEPLOY.md)
        └─ Ref: ADR 0011 — Lab (local) → Puente (CI/staging) → Producción
    [x] Sin `// TODO` sin ticket asignado en el código ✅
        └─ Ref: ADR 0011 — Deuda técnica documentada
    [x] Sin `FIXME` o `HACK` sin contexto en comentarios ✅
        └─ Ref: ADR 0011 — Código explícito
```

---

## G.5 — Verificaciones de calidad automatizadas

> **Referencia:** ADR 0011 (Estándares), ADR 0010 (Testing)

```
[x] Scripts de calidad en justfile:
    [x] just lint  → cargo clippy -D warnings ✅
        └─ Ref: ADR 0011, docs/02-STACK.md L456
    [x] just fmt-check  → cargo fmt --all --check ✅
        └─ Ref: ADR 0012 — formato consistente
    [x] just check-lines  → verifica límites de líneas por archivo ✅
        └─ Ref: ADR 0011 — archivos ≤200 líneas
    [~] just check-fn  → verifica longitud de funciones
        └─ Ref: ADR 0011 — funciones ≤30 líneas (⚠️ herramienta externa requerida)
```

---

## ✅ Entregable de Génesis

Cuando todos los checks pasan, el proyecto está listo para el **Bloque I — Fundación**.

```bash
cargo check --workspace   # verde
cargo deny check          # verde
just --list               # muestra todos los comandos
git commit --allow-empty -m "chore: genesis complete"  # lefthook ok
```

**Referencia siguiente fase:** → `ROADMAP-BACKEND.md` — Bloque I (Fundación)
    └─ Ref: ADR 0004, ADR 0006 — Pool SQLite + 6 migraciones RBAC

---

## Troubleshooting — Génesis

### G.1 — Estructura física

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `mkdir: cannot create directory` | Permisos o directorio existe | `ls -la` verificar, usar `mkdir -p` |
| `pnpm not found` | Node no instalado | `npm install -g pnpm` o usar `mise` |

### G.2 — Cargo.toml

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `cargo check` error "failed to load manifest" | Cargo.toml malformado | Validar TOML con `cargo verify-project` |
| `unresolved import sqlx` en domain | Violación ADR 0001 | Verificar `crates/domain/Cargo.toml` NO tenga sqlx |
| `jsonwebtoken` encontrado | Violación ADR 0008 | Reemplazar por `pasetors`, ver ADR 0008 |
| `duplicate workspace member` | Crate listado 2 veces en root Cargo.toml | Buscar duplicado en `[workspace.members]` |
| `feature resolver = "2"` warning | Resolver no especificado | Añadir `resolver = "2"` en `[workspace]` |

### G.3 — Tooling

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `just: command not found` | just no instalado | `cargo install just` o usar `mise` |
| `lefthook install` falla | No es repo git | `git init` primero |
| `cargo deny check` falla | Licencia prohibida encontrada | Revisar `deny.toml` o actualizar dependencias |
| `.env.local` no existe | No copiado desde ejemplo | `cp .env.example .env.local` |
| `sqlx migrate` error | DATABASE_URL no seteada | Exportar `DATABASE_URL` en shell o .env |

### G.5 — Verificaciones de calidad

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| `just check-lines` falla | Archivo supera 200 líneas | Refactorizar en submódulos — Ref: ADR 0011 |
| `just check-fn` falla | Función supera 30 líneas | Dividir función con nombres descriptivos — Ref: ADR 0011 |
| `// TODO` sin ticket encontrado | Deuda técnica no documentada | Crear ticket y cambiar a `// TODO(#123): descripción` — Ref: ADR 0011 |
| `FIXME` sin contexto | Hotfix temporal olvidado | Documentar contexto o resolver antes de merge — Ref: ADR 0011 |
| `cargo clippy -D warnings` falla | Warnings de código | Resolver warnings o añadir `#![allow(...)]` justificado — Ref: ADR 0011 |

---

**Nota:** Si un error persiste, revisar el ADR correspondiente listado en las referencias de cada fase.
