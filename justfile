# Ubicación: `justfile`
#
# Descripción: Comandos de desarrollo para el monorepo.
#              Usa `just --list` para ver todos los comandos.
#
# ADRs relacionados: ADR 0012 (Herramientas)
# Versión: 2026.04

# Carga automática de variables desde .env
set dotenv-load := true

# Shell configuration para Windows (Git Bash / MINGW64)
set windows-shell := ["sh", "-c"]

# Default: mostrar ayuda
_default:
    @just --list

# ──────────────────────────────────────────────────────────────────────────────
# Setup y Verificación
# ──────────────────────────────────────────────────────────────────────────────

# Verifica que todo el toolchain esté instalado
[no-cd]
doctor:
    @echo "🔍 Verificando toolchain..."
    rustc --version
    cargo --version
    mise --version 2>/dev/null || echo "⚠️  mise no instalado"
    pnpm --version 2>/dev/null || echo "⚠️  pnpm no instalado"
    just --version
    @echo "✅ Toolchain verificado"

# Instala todo: dependencias, hooks, herramientas
[no-cd]
setup:
    @echo "🔧 Configurando entorno..."
    cargo install cargo-watch --version 0.8.29 2>/dev/null || true
    cargo install cargo-nextest --version 0.9.132 2>/dev/null || true
    cargo install cargo-deny --version 0.18.2 2>/dev/null || true
    cargo install cargo-audit --version 0.21.2 2>/dev/null || true
    cargo install sqlx-cli --version 0.8.3 --features sqlite 2>/dev/null || true
    cargo install lefthook --version 2.1.5 2>/dev/null || true
    @echo "✅ Setup completo"

# ──────────────────────────────────────────────────────────────────────────────
# Desarrollo
# ──────────────────────────────────────────────────────────────────────────────

# Hot reload del API
[no-cd]
dev-api:
    cargo watch -p api -x "run --bin api"

# Desarrollo completo (API + futuro frontend)
[no-cd]
dev:
    @echo "🚀 Iniciando desarrollo..."
    just dev-api &
    @echo "✅ Dev iniciado"

# ──────────────────────────────────────────────────────────────────────────────
# Build
# ──────────────────────────────────────────────────────────────────────────────

# Compila solo el binario API
[no-cd]
build-api:
    cargo build --release --bin api

# Compila todo el workspace
[no-cd]
build:
    cargo build --release

# ──────────────────────────────────────────────────────────────────────────────
# Testing
# ──────────────────────────────────────────────────────────────────────────────

# Ejecuta tests con nextest (capas 1-3)
[no-cd]
test:
    cargo nextest run --no-tests=pass

# Ejecuta todos los tests incluyendo E2E
[no-cd]
test-all:
    cargo nextest run --all-targets

# Ejecuta tests con output verbose
[no-cd]
test-v:
    cargo nextest run --no-capture

# ──────────────────────────────────────────────────────────────────────────────
# Calidad de Código
# ──────────────────────────────────────────────────────────────────────────────

# Lint con clippy
[no-cd]
lint:
    cargo clippy --workspace -- -D warnings

# Formatea todo el código
[no-cd]
fmt:
    cargo fmt --all

# Verificación rápida (check)
[no-cd]
check:
    cargo check --workspace

# Verifica formato sin modificar
[no-cd]
fmt-check:
    cargo fmt --all --check

# Verifica líneas por archivo (≤200 líneas)
[no-cd]
check-lines:
    @echo "Verificando archivos ≤200 líneas..."
    @find crates -name "*.rs" -exec sh -c 'lines=$(wc -l < "$1"); if [ $lines -gt 200 ]; then echo "OVER: $1 = $lines líneas"; exit 1; fi' _ {} \;
    @find apps -name "*.rs" -exec sh -c 'lines=$(wc -l < "$1"); if [ $lines -gt 200 ]; then echo "OVER: $1 = $lines líneas"; exit 1; fi' _ {} \;
    @echo "✅ Todos los archivos ≤200 líneas"

# Auditoría de seguridad y licencias
[no-cd]
audit:
    cargo deny check
    cargo audit

# ──────────────────────────────────────────────────────────────────────────────
# Base de Datos
# ──────────────────────────────────────────────────────────────────────────────

# Fallback DATABASE_URL si no está configurado
export DATABASE_URL := env_var_or_default("DATABASE_URL", "sqlite:./data/boilerplate.db")

# Ejecuta migraciones pendientes
[no-cd]
migrate:
    @echo "DATABASE_URL=$DATABASE_URL"
    @mkdir -p data
    @test -f data/boilerplate.db || sqlite3 data/boilerplate.db "SELECT 1;"
    sqlx migrate run --source data/migrations

# Reset de base de datos (cuidado!)
[no-cd]
migrate-reset:
    sqlx migrate reset --source data/migrations

# Crea nueva migración
[no-cd]
migrate-new name:
    sqlx migrate add --source data/migrations {{name}}

# Estado de migraciones
[no-cd]
db-status:
    sqlx migrate info --source data/migrations

# Prepara queries para modo offline
[no-cd]
prepare:
    cargo sqlx prepare --workspace

# ──────────────────────────────────────────────────────────────────────────────
# Git Hooks
# ──────────────────────────────────────────────────────────────────────────────

# Instala hooks de git
[no-cd]
hooks:
    lefthook install

# Ejecuta hooks manualmente
[no-cd]
hooks-run:
    lefthook run pre-commit

# ──────────────────────────────────────────────────────────────────────────────
# Utilidades
# ──────────────────────────────────────────────────────────────────────────────

# Limpia archivos de build
[no-cd]
clean:
    cargo clean

# Actualiza dependencias (cuidado!)
[no-cd]
update:
    cargo update

# Muestra árbol de dependencias
[no-cd]
tree:
    cargo tree --depth 2
