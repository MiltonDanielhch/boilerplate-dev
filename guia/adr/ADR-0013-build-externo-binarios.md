# ADR 0013 — Build Externo: Distroless ~10MB

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0014 (Containerfile + Kamal), ADR 0010 (CI), ADR 0012 (just deploy) |

---

## Contexto

El VPS de producción ($5, 1GB RAM) tiene recursos muy limitados. Compilar un proyecto Rust
con Axum, SQLx y Apalis en el servidor:

- Consume >2GB de RAM — causaría caídas de los servicios activos
- Tarda 5-15 minutos — agota los créditos de CPU del VPS
- Requiere instalar Rust toolchain en producción — superficie de ataque innecesaria

---

## Decisión

**Prohibir la compilación en el servidor de producción.** El binario se compila en local
o en CI y se envía al servidor ya compilado y optimizado.

### 1 — Compilación cruzada a binario estático

```bash
# En local o CI — nunca en el VPS
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
# → Binario ~8-12MB completamente autónomo sin dependencias de libc
```

### 2 — Containerfile multi-stage con distroless

```dockerfile
# infra/docker/Containerfile

# ── Stage 1: Build ──────────────────────────────────────────────────────────
FROM rust:1.82-slim AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y musl-tools pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY crates/  ./crates/
COPY apps/api/ ./apps/api/

# SQLX_OFFLINE=true — el build no necesita acceso a la DB
ENV SQLX_OFFLINE=true
RUN cargo build --release --target x86_64-unknown-linux-musl --bin api

# ── Stage 2: Runtime mínimo ──────────────────────────────────────────────────
# Sin shell, sin package manager, sin herramientas de desarrollo
FROM gcr.io/distroless/cc-debian12

COPY --from=builder \
    /app/target/x86_64-unknown-linux-musl/release/api /api
COPY --from=builder /app/data/migrations /migrations

COPY --from=ghcr.io/benbjohnson/litestream:latest-amd64 \
    /usr/local/bin/litestream /litestream
COPY infra/litestream/litestream.yml /etc/litestream.yml

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["/api", "health"]

EXPOSE 8080
ENTRYPOINT ["/litestream", "replicate", "-exec", "/api"]
```

El builder nunca llega al servidor — solo la imagen final de ~10MB.

### 3 — Optimizaciones del binario en release

```toml
# Cargo.toml
[profile.release]
opt-level     = "z"     # Optimizar por tamaño — binario más pequeño
lto           = true    # Link Time Optimization — elimina código muerto
codegen-units = 1       # Mejor optimización a costa de tiempo de build local
panic         = "abort" # Sin stack unwinding — ahorra espacio
strip         = true    # Elimina símbolos de debug del binario final
```

| Métrica | Sin optimizar | Con optimizar |
|---------|--------------|---------------|
| Tamaño del binario | ~25MB | ~8-12MB |
| RAM en reposo | ~45MB | ~30MB |
| Tiempo de arranque | ~200ms | ~80ms |

### 4 — `SQLX_OFFLINE=true` en el build

```bash
# Antes de commitear cambios de SQL:
just prepare   # → cargo sqlx prepare --workspace
# Genera/actualiza .sqlx/ en git — el build funciona sin DB

# CI verifica que .sqlx/ está actualizado:
just prepare && git diff --exit-code .sqlx/
```

### 5 — Flujo de deploy

```makefile
# justfile — el deploy NUNCA toca el servidor para compilar
deploy:
    just audit        # cargo deny check + cargo audit
    just test         # cargo nextest run
    kamal deploy      # build local → push registry → pull VPS → swap
    # Kamal: descarga ~10MB en lugar de compilar 15 minutos
```

---

## Comparativa

| Estrategia | Tiempo deploy | RAM VPS | Seguridad |
|-----------|--------------|---------|-----------|
| Compilar en VPS | 10-15 min | >2GB (mata servicios) | Código fuente expuesto |
| Binario por SCP | 30s | ~0 | Solo binario |
| **Kamal + registry** | **2-3 min** | **~0** | **Sin código fuente** |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la eficiencia del binario y la simplicidad del pipeline:

| Herramienta | Propósito en el Build |
| :--- | :--- |
| **`cargo-zigbuild`** | **Compilación Cruzada:** Facilita la creación de binarios estáticos para Linux desde cualquier OS usando Zig como linker. |
| **`cargo-bloat`** | **Análisis de Tamaño:** Identifica qué secciones del código o dependencias están ocupando más espacio en el binario. |
| **`upx`** | **Compresión Extrema:** Reduce el tamaño del binario final hasta un 60-70% adicional (opcional para entornos ultra-limitados). |
| **`cargo-dist`** | **Release Pipeline:** Automatiza la generación de artefactos y la subida a registros de contenedores. |

---

## Consecuencias

### ✅ Positivas

- El VPS mantiene RAM libre para usuarios durante el deploy
- Sin código fuente en producción — superficie de ataque mínima
- Deploy reproducible — el mismo binario en staging y producción
- `SQLX_OFFLINE=true` garantiza que el build funciona sin DB en CI

### ⚠️ Negativas / Trade-offs

- La máquina local necesita el toolchain de compilación cruzada
  → `just setup` instala todo incluyendo el target musl
  → En CI: GitHub Actions con caché de `~/.cargo/registry` y `target/`
- `just prepare` debe correr antes de commitear cambios de SQL
  → Incluido en el pre-push hook de lefthook (ADR 0012)
  → El CI falla con error claro si `.sqlx/` no está actualizado
- El primer build local tarda más por `lto=true` y `codegen-units=1`
  → Builds posteriores son incrementales — solo recompilan lo que cambió
  → En CI: cachear `target/` entre runs reduce el tiempo a 2-3 minutos

### Decisiones derivadas

- `.sqlx/` se incluye en git — permite builds sin DB en CI y Containerfile
- `just prepare` está en el pre-push hook de lefthook (ADR 0012)
- El Containerfile siempre usa `gcr.io/distroless/cc-debian12` — no Alpine, no Ubuntu
- La imagen final nunca supera 15MB — si supera, investigar con `cargo bloat`
