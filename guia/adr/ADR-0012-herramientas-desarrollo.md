# ADR 0012 — Herramientas: just + pnpm + lefthook

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0014 (just deploy como comando de despliegue) |

---

## Contexto

Sin un estándar de tooling, cada developer aprende los comandos del proyecto de forma ad-hoc,
se olvida de correr lint o audit antes de commitear, y el onboarding es innecesariamente largo.

Necesitamos:

- Un runner de comandos único que funcione igual en Linux, macOS y Windows
- Gestión de paquetes JS eficiente con workspaces nativos
- Garantía de que nadie suba código que no pasa calidad mínima
- Un comando de setup que deje el entorno listo en menos de 5 minutos

---

## Decisión

**just** como runner de comandos, **pnpm** para paquetes JS, **lefthook** para git hooks.

### Por qué just en lugar de Makefile

Makefile tiene problemas reales para un monorepo mixto Rust/JS: tabs obligatorios que generan
errores silenciosos, sintaxis de variables diferente a bash, y sin soporte nativo para argumentos
nombrados. `just` resuelve todo eso con una sintaxis limpia y predecible.

### justfile completo

```makefile
# justfile — boilerplate
# Ver todos los comandos: just --list

# ── Setup ──────────────────────────────────────────────────────────────────────

setup:
    cargo install cargo-watch cargo-nextest cargo-deny cargo-audit sqlx-cli lefthook
    npm install -g pnpm
    pnpm install
    cp -n .env.example .env.local || true
    lefthook install
    sqlx database create
    just migrate
    @echo "✅ Setup completo. Edita .env.local y ejecuta: just dev"

# ── Desarrollo ─────────────────────────────────────────────────────────────────

dev:
    cargo watch -x "run --bin api" & pnpm --filter web dev

dev-api:
    cargo watch -x "run --bin api"

dev-web:
    pnpm --filter web dev

# ── Build ──────────────────────────────────────────────────────────────────────

build:
    pnpm --filter mailer build
    cargo build --release
    just types
    pnpm --filter web build

# ── Tests ──────────────────────────────────────────────────────────────────────

test:
    cargo nextest run

test-all:
    cargo nextest run --all-targets

test-v:
    cargo nextest run --no-capture

# ── Calidad ────────────────────────────────────────────────────────────────────

lint:
    cargo clippy --all-targets -- -D warnings
    pnpm --filter web lint

fmt:
    cargo fmt --all
    pnpm --filter web format

check:
    cargo check --workspace

audit:
    cargo deny check
    cargo audit

# ── Base de datos ──────────────────────────────────────────────────────────────

migrate:
    sqlx migrate run

migrate-reset:
    sqlx database reset

migrate-new name:
    sqlx migrate add {{name}}

db-status:
    sqlx migrate info

prepare:
    cargo sqlx prepare --workspace

# ── Tipos TypeScript (buf generate — ADR 0027) ─────────────────────────────────

types:
    buf generate
    @echo "✅ Tipos generados en apps/web/src/lib/types/api.ts"

types-check:
    buf generate
    git diff --exit-code apps/web/src/lib/types/api.ts

# ── Deploy ─────────────────────────────────────────────────────────────────────

deploy-setup:
    kamal setup

deploy:
    just audit
    just test
    kamal deploy
    curl -fsS ${HC_DEPLOY_UUID:+https://hc-ping.com/$HC_DEPLOY_UUID} || true

redeploy:
    kamal redeploy

rollback:
    kamal rollback

# ── Utilidades ─────────────────────────────────────────────────────────────────

db-prod:
    kamal app exec "sqlite3 /data/boilerplate.db '.tables'"

logs:
    kamal logs -f

status:
    kamal details
```

### pnpm — workspaces

```yaml
# pnpm-workspace.yaml
packages:
  - 'apps/web'
  - 'apps/mailer'
```

### lefthook — git hooks que no se pueden saltar

```yaml
# lefthook.yml
pre-commit:
    parallel: true
    commands:
        fmt-rust:
            run:  cargo fmt --all --check
            glob: "*.rs"
        check:
            run:  cargo check --workspace

pre-push:
    commands:
        lint:
            run: cargo clippy --all-targets -- -D warnings
        test:
            run: cargo nextest run
        audit:
            run: cargo deny check
```

```bash
# Instalación — incluida en just setup
lefthook install

# Saltarse los hooks en emergencias (trazable en git log)
LEFTHOOK=0 git push
```

### Onboarding de un developer nuevo

```bash
git clone https://github.com/tuuser/boilerplate
cd boilerplate
just setup

# Output esperado en <5 minutos:
# ✅ cargo tools instaladas
# ✅ pnpm install completado
# ✅ .env.local creado desde .env.example
# ✅ lefthook hooks registrados
# ✅ DB creada y 6 migraciones aplicadas
# ✅ Setup completo. Edita .env.local y ejecuta: just dev
```

---

## Alternativas consideradas

| Herramienta | Motivo de descarte |
|-------------|-------------------|
| Makefile | Tabs obligatorios, sintaxis confusa, poca legibilidad en monorepos |
| npm scripts | Solo JS — no integra comandos Rust/bash de forma limpia |
| Taskfile (go-task) | Requiere instalar Go o descargar un binario externo |
| husky | Solo Node.js — no funciona bien en monorepos mixtos Rust/JS |
| pre-commit (Python) | Dependencia de Python innecesaria en un stack Rust/JS |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la automatización y la consistencia del entorno:

| Herramienta | Propósito en el Desarrollo |
| :--- | :--- |
| **`mise`** | **Gestor de Toolchain:** Administra versiones de Rust, Node, pnpm y herramientas CLI de forma automática por proyecto. |
| **`bacon`** | **Background Checker:** Compilación y ejecución de tests en segundo plano con feedback visual inmediato. |
| **`typos`** | **Corrector Ortográfico:** Evita errores en código y documentación que confunden a humanos y agentes IA. |
| **`cargo-dist`** | **Pipeline de Release:** Automatiza el empaquetado de binarios para despliegue y distribución multiplataforma. |

---

## Consecuencias

### ✅ Positivas

- `just setup` deja el entorno listo en <5 minutos para cualquier developer
- `just --list` documenta todos los comandos disponibles — la documentación vive en el código
- lefthook previene commits con errores de formato o lint
- pnpm workspaces gestiona las dependencias JS del monorepo de forma eficiente

### ⚠️ Negativas / Trade-offs

- `just` y `lefthook` son menos conocidos que Makefile y husky
  → `just --help` y `lefthook --help` son suficientes para aprender lo esencial
  → La guía INICIO.md incluye los 5 comandos que se usan el 90% del tiempo
- El pre-push hook puede sentirse lento si los tests tardan
  → `LEFTHOOK=0` está documentado como escape de emergencia — no como práctica normal
  → `just test` corre en <5 segundos (capas 1-3); el pre-push hook usa la misma suite

### Decisiones derivadas

- `just deploy` incluye `just audit` y `just test` — no se puede hacer deploy sin pasar calidad
- `lefthook install` es parte de `just setup` — obligatorio desde el primer commit
- `just prepare` debe correr antes de commitear cambios de queries SQL (actualiza `.sqlx/`)
- `types` usa `buf generate` (ADR 0027) — en Fase 1 genera los tipos desde los endpoints REST
