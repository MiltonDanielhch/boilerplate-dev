# Roadmap — Infraestructura y Deploy

> **Stack:** Podman rootless · Caddy · Kamal · Litestream · Tigris S3 · Healthchecks.io
>
> **ADRs:** 0013 (Build) · 0014 (Deploy) · 0004 (Litestream) · 0015 (Monitoreo) · 0010 (CI)
>
> **Pre-requisitos:** Backend Bloques I-VI completados (el MVP debe compilar limpio)

---

## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
```

---

## Progreso

| Bloque | Nombre | Progreso |
|--------|--------|----------|
| INF.I | Containerfile distroless | 0% |
| INF.II | Caddy — TLS + seguridad | 0% |
| INF.III | Litestream + S3 | 0% |
| INF.IV | Kamal — deploy zero-downtime | 0% |
| INF.V | Seguridad del VPS | 0% |
| INF.VI | Monitoreo y alertas | 0% |
| INF.VII | CI/CD | 0% |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la seguridad, simplicidad y visibilidad del despliegue:

| Herramienta | Propósito en la Infra |
| :--- | :--- |
| **`kamal-proxy`** | **Simplificación:** El proxy nativo de Kamal 2 que gestiona TLS y tráfico sin Caddy externo. |
| **`CrowdSec`** | **Defensa Activa:** Bloqueo colaborativo de IPs maliciosas y ataques de fuerza bruta. |
| **`Sentry Crons`** | **Monitoreo Unificado:** Centraliza los pings de Apalis y Litestream junto con los logs de error. |
| **`cargo-zigbuild`** | **Build Rápido:** Compilación cruzada eficiente para generar el binario de Linux desde cualquier OS. |

---

## INF.I — Containerfile multi-stage distroless (ADR 0013, 0014)

> **Referencia:** ADR 0013 (Build), ADR 0014 (Deploy), ADR 0004 (Litestream), docs/02-STACK.md L155-170, L360-368

```
[ ] infra/docker/Containerfile:
    └─ Ref: docs/03-STRUCTURE.md L555-558

    [ ] Stage 1 — builder:
        [ ] FROM rust:1.85-slim AS builder
            └─ Ref: ADR 0013, docs/02-STACK.md L413, rust-toolchain.toml
        [ ] apt-get install musl-tools pkg-config
        [ ] rustup target add x86_64-unknown-linux-musl
        [ ] COPY Cargo.toml Cargo.lock
        [ ] COPY crates/ apps/api/
        [ ] ENV SQLX_OFFLINE=true
            └─ Ref: ADR 0013 — offline mode para CI
        [ ] cargo build --release --target x86_64-unknown-linux-musl --bin api

    [ ] Stage 2 — runtime:
        [ ] FROM gcr.io/distroless/cc-debian12  (NO Alpine, NO Ubuntu)
            └─ Ref: ADR 0013, ADR 0014 — distroless por seguridad
        [ ] COPY binario /api
        [ ] COPY data/migrations /migrations
        [ ] COPY Litestream desde ghcr.io/benbjohnson/litestream:latest-amd64
            └─ Ref: ADR 0004, docs/02-STACK.md L165-170
        [ ] COPY infra/litestream/litestream.yml /etc/litestream.yml
        [ ] HEALTHCHECK CMD ["/api", "health"]
        [ ] EXPOSE 8080
        [ ] ENTRYPOINT ["/litestream", "replicate", "-exec", "/api"]
            └─ Ref: ADR 0004 — Litestream como entrypoint

[ ] Verificar imagen:
    └─ Ref: ADR 0013, ADR 0014
    [ ] podman build -f infra/docker/Containerfile -t boilerplate:local .
    [ ] podman image ls boilerplate:local → tamaño < 15MB
        └─ Ref: ADR 0013 — imagen minimalista
    [ ] podman run --rm boilerplate:local sh → falla (sin shell ✓)
        └─ Ref: ADR 0013 — distroless sin shell
    [ ] podman run --rm -p 8080:8080 boilerplate:local
    [ ] curl http://localhost:8080/health → {"status":"ok"}
```

---

## INF.II — Caddy (ADR 0014)

> **Referencia:** ADR 0014 (Deploy), docs/02-STACK.md L360-368

```
[ ] infra/caddy/Caddyfile:
    └─ Ref: docs/03-STRUCTURE.md L559-562

    [ ] tudominio.com {
            reverse_proxy localhost:8080
            encode gzip zstd
            header {
                Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
                X-Content-Type-Options    "nosniff"
                X-Frame-Options           "DENY"
                Referrer-Policy           "strict-origin-when-cross-origin"
                Permissions-Policy        "geolocation=(), microphone=(), camera=()"
                -Server
            }
            @static path /assets/* /images/* /fonts/*
            header @static Cache-Control "public, max-age=31536000, immutable"
            log { output file /var/log/caddy/access.log; format json }
        }
        └─ Ref: ADR 0014 — headers de seguridad

    [ ] app.tudominio.com {
            reverse_proxy localhost:4321
            encode gzip zstd
            header {
                Strict-Transport-Security "max-age=31536000; includeSubDomains"
                X-Content-Type-Options    "nosniff"
                X-Frame-Options           "DENY"
                -Server
            }
        }

[ ] Verificar:
    [ ] TLS automático Let's Encrypt funciona
        └─ Ref: ADR 0014, docs/02-STACK.md L362
    [ ] HTTP redirige a HTTPS
    [ ] curl -I https://tudominio.com → Strict-Transport-Security presente
    [ ] curl -I https://tudominio.com → no muestra header Server
        └─ Ref: ADR 0014 — ocultar Server header
    [ ] curl https://tudominio.com/health → {"status":"ok"}
```

---

## INF.III — Litestream + S3/Tigris (ADR 0004, 0020)

> **Referencia:** ADR 0004 (Litestream), ADR 0020 (Storage S3), docs/02-STACK.md L155-170, L320-332

```
[ ] Credenciales Tigris en .env:
    └─ Ref: ADR 0020, docs/02-STACK.md L320-332
    [ ] AWS_ENDPOINT_URL_S3=https://fly.storage.tigris.dev
    [ ] AWS_ACCESS_KEY_ID=tid_xxx
    [ ] AWS_SECRET_ACCESS_KEY=tsec_xxx
    [ ] LITESTREAM_BUCKET=boilerplate-production-backups
        └─ Ref: ADR 0004, docs/02-STACK.md L168
    [ ] STORAGE_BUCKET=boilerplate-production-assets  (para uploads de usuarios)
        └─ Ref: ADR 0020

[ ] infra/litestream/litestream.yml:
    └─ Ref: docs/03-STRUCTURE.md L563-566
    [ ] path: /data/boilerplate.db
    [ ] replicas:
        [ ] type: s3
        [ ] bucket: ${LITESTREAM_BUCKET}
        [ ] endpoint: ${AWS_ENDPOINT_URL_S3}
        [ ] sync-interval: 1s       → RPO ~1 segundo
            └─ Ref: docs/02-STACK.md L168 — 1s sync
        [ ] snapshot-interval: 24h  → snapshot diario
        [ ] retention: 72h          → 3 días de WAL
            └─ Ref: docs/02-STACK.md L169 — retención WAL

[ ] Verificar replicación:
    └─ Ref: ADR 0004
    [ ] litestream snapshots s3://bucket/boilerplate/db → entradas recientes
    [ ] Verificar que sync-interval funciona: crear registro → ver en S3 en <5s

[ ] Test de restauración (CRÍTICO — probar ANTES de producción):
    └─ Ref: ADR 0004, docs/02-STACK.md L170
    [ ] Simular pérdida de VPS
    [ ] litestream restore -o /data/boilerplate.db s3://bucket/boilerplate/db
    [ ] sqlite3 /data/boilerplate.db "PRAGMA integrity_check;" → ok
    [ ] just deploy → sistema funciona con datos restaurados
```

---

## INF.IV — Kamal zero-downtime (ADR 0014)

> **Referencia:** ADR 0014 (Deploy), docs/02-STACK.md L360-368, docs/03-STRUCTURE.md L567-570

```
[ ] infra/kamal/deploy.yml:
    └─ Ref: docs/03-STRUCTURE.md L567-570
    [ ] service: boilerplate
    [ ] image: ghcr.io/tuuser/boilerplate
    [ ] servers: [IP del VPS]
    [ ] registry: ghcr.io + KAMAL_REGISTRY_PASSWORD
    [ ] env.clear: PORT=8080, RUST_LOG=info,sqlx=warn, ENVIRONMENT=production
    [ ] env.secret:
        [ ] DATABASE_URL
        [ ] PASETO_SECRET
        [ ] RESEND_API_KEY
            └─ Ref: ADR 0019
        [ ] AWS_ACCESS_KEY_ID
        [ ] AWS_SECRET_ACCESS_KEY
        [ ] AWS_ENDPOINT_URL_S3
        [ ] LITESTREAM_BUCKET
            └─ Ref: ADR 0004, docs/02-STACK.md L168
        [ ] SENTRY_DSN
            └─ Ref: ADR 0016
    [ ] volumes: ["/data/boilerplate:/data"]
    [ ] healthcheck: { path: /health, port: 8080, max_attempts: 10, interval: 3s }
        └─ Ref: ADR 0014, docs/02-STACK.md L366
    [ ] ssh: { user: deploy, keys_only: true }
        └─ Ref: ADR 0014 — SSH solo con clave

[ ] Comandos:
    [ ] kamal setup → primera configuración del VPS
    [ ] just deploy → audit + test + kamal deploy
        └─ Ref: docs/02-STACK.md L360-368
    [ ] kamal rollback → rollback en ~5 segundos
        └─ Ref: ADR 0014

[ ] Verificar zero-downtime:
    └─ Ref: ADR 0014
    [ ] Iniciar loop de requests: while true; do curl -s /health; sleep 0.5; done
    [ ] En otro terminal: kamal redeploy
    [ ] Verificar: todos los status son 200, ninguno es 502/503

[ ] Verificar rollback:
    └─ Ref: ADR 0014
    [ ] kamal rollback
    [ ] Tiempo de rollback < 10 segundos
    [ ] Healthcheck sigue respondiendo 200
```

---

## INF.V — Seguridad del VPS

> **Referencia:** ADR 0014 (Deploy), docs/02-STACK.md L360-368

```
[ ] Usuario deploy sin root:
    └─ Ref: ADR 0014
    [ ] adduser deploy
    [ ] usermod -aG systemd-journal deploy  (para ver logs)

[ ] SSH solo con clave pública:
    └─ Ref: ADR 0014
    [ ] PasswordAuthentication no  (en /etc/ssh/sshd_config)
    [ ] PermitRootLogin no
    [ ] systemctl restart sshd

[ ] Firewall UFW mínimo:
    └─ Ref: ADR 0014
    [ ] ufw allow ssh
    [ ] ufw allow 80
    [ ] ufw allow 443
    [ ] ufw enable
    [ ] ufw status → solo puertos necesarios

[ ] Actualizaciones automáticas de seguridad:
    └─ Ref: ADR 0014
    [ ] apt install unattended-upgrades
    [ ] dpkg-reconfigure unattended-upgrades

[ ] Verificar: ssh root@vps → rechazado
    └─ Ref: ADR 0014
[ ] Verificar: ssh deploy@vps → funciona con clave
[ ] Verificar: acceso a puerto 8080 directo → rechazado (UFW)
    └─ Ref: ADR 0014 — solo Caddy expuesto
```

---

## INF.VI — Monitoreo y alertas (ADR 0015)

> **Referencia:** ADR 0015 (Monitoreo), ADR 0016 (Observabilidad), docs/02-STACK.md L335-358

```
[ ] Healthchecks.io — crear checks:
    └─ Ref: ADR 0015, docs/02-STACK.md L348
    [ ] Check de Litestream: período 1h, grace 15min
        └─ Ref: ADR 0004
    [ ] Check de Apalis: período 5min, grace 2min
        └─ Ref: ADR 0018
    [ ] Check de TLS cert: período 24h, grace 2h
    [ ] Check de deploy: manual (ping en just deploy)

[ ] Variables en .env:
    └─ Ref: ADR 0015, docs/02-STACK.md L348
    [ ] HC_LITESTREAM_UUID=xxx
    [ ] HC_APALIS_UUID=xxx
    [ ] HC_TLS_UUID=xxx
    [ ] HC_DEPLOY_UUID=xxx

[ ] Integración en código:
    └─ Ref: ADR 0015, docs/02-STACK.md L348
    [ ] CleanupJob hace ping a HC_APALIS_UUID al completar
        └─ Ref: ADR 0018
    [ ] Litestream script hace ping a HC_LITESTREAM_UUID
        └─ Ref: ADR 0004, docs/02-STACK.md L167
    [ ] just deploy hace ping a HC_DEPLOY_UUID como último paso
        └─ Ref: ADR 0015

[ ] Alertas configuradas para:
    [ ] Litestream → Email + Telegram (inmediato)
    [ ] Apalis → Email urgente
    [ ] TLS → Email

[ ] Verificar alertas:
    [ ] Detener el servidor → Healthchecks.io alerta en ~5min
    [ ] Restaurar el servidor → alerta de recuperación
```

---

## INF.VII — CI/CD (ADR 0010, 0013)

> **Referencia:** ADR 0010 (Testing), ADR 0013 (Build), docs/02-STACK.md L429-443, L413-415

```
[ ] .github/workflows/ci.yml:
    └─ Ref: docs/03-STRUCTURE.md L571-573
    [ ] Trigger: push a main, PR a main
    [ ] Cache: ~/.cargo/registry + ~/.cargo/git + target/
        └─ Ref: docs/02-STACK.md L443
    [ ] Steps:
        [ ] cargo nextest run --all-targets
            └─ Ref: ADR 0010, docs/02-STACK.md L442-443
        [ ] cargo clippy --all-targets -- -D warnings
            └─ Ref: ADR 0013
        [ ] cargo deny check
            └─ Ref: docs/02-STACK.md L412 — prohibe jwt, unsafe, etc.
        [ ] cargo audit
            └─ Ref: ADR 0013
        [ ] just types-check  → falla si api.ts tiene diff
            └─ Ref: ADR 0027
        [ ] just prepare      → falla si .sqlx/ no está actualizado
            └─ Ref: ADR 0013, docs/02-STACK.md L415
        [ ] Build imagen distroless (verificar que compila)
            └─ Ref: ADR 0013

[ ] Verificar CI:
    └─ Ref: ADR 0010
    [ ] Push a main → CI corre
    [ ] PR con error de clippy → CI falla
    [ ] PR con CVE en dependencia → CI falla
        └─ Ref: ADR 0013
    [ ] PR con jsonwebtoken en Cargo.toml → cargo deny falla
        └─ Ref: ADR 0008 — JWT prohibido

[ ] SQLX_OFFLINE=true activo en CI:
    └─ Ref: ADR 0013, docs/02-STACK.md L415
    [ ] just prepare generado y en git
    [ ] Containerfile usa ENV SQLX_OFFLINE=true
    [ ] Build en CI no necesita DB activa
```

---

## Verificación final — MVP en producción

```bash
# 1. Deploy exitoso
just deploy
# → audit ✓, tests ✓, kamal deploy ✓, HC ping ✓

# 2. HTTPS funcionando
curl -I https://tudominio.com/health
# → 200, Strict-Transport-Security presente

# 3. Zero-downtime verificado
# while loop + kamal redeploy → ningún 502

# 4. Rollback funciona
kamal rollback
# → < 10 segundos, healthcheck sigue verde

# 5. Backup existe
litestream snapshots s3://bucket/boilerplate/db
# → entradas de hoy

# 6. Sin shell en producción
kamal app exec "sh"
# → error: exec format error (distroless sin shell ✓)

# 7. Healthchecks.io tiene pings
# → Dashboard de Healthchecks.io muestra todos los checks verdes
```

---

## Diagrama de Flujo de Infraestructura

```
┌─────────────────────────────────────────────────────────────────────────┐
│  INF.I — CONTAINERFILE DISTROLESS                                      │
│  ├─ Stage 1: rust:1.82-slim (builder)                                  │
│  ├─ Stage 2: gcr.io/distroless/cc-debian12 (runtime)                   │
│  ├─ SQLX_OFFLINE=true para CI                                          │
│  ├─ Litestream incluido en imagen                                      │
│  └─ Entrypoint: /litestream replicate -exec /api                       │
│     └─ Ref: ADR 0013, 0014, 0004                                      │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  INF.II — CADDY (Reverse Proxy + TLS)                                  │
│  ├─ tudominio.com → reverse_proxy localhost:8080                       │
│  ├─ TLS automático Let's Encrypt                                       │
│  ├─ Headers de seguridad (HSTS, X-Frame-Options, etc.)                 │
│  ├─ Cache estático (assets, images, fonts)                             │
│  └─ Logs JSON para observabilidad                                      │
│     └─ Ref: ADR 0014                                                  │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  INF.III — LITESTREAM + S3 (Backup Continuo)                           │
│  ├─ Replicación WAL cada 1s a S3/Tigris                                │
│  ├─ Snapshots diarios (24h interval)                                   │
│  ├─ Retención 72h de WAL                                               │
│  └─ Healthcheck ping a Healthchecks.io                                 │
│     └─ Ref: ADR 0004, 0020                                             │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  INF.IV — KAMAL (Deploy Zero-Downtime)                                 │
│  ├─ Config: deploy.yml con servers, registry, secrets                  │
│  ├─ Comandos: setup, deploy, rollback                                  │
│  ├─ Healthcheck antes de switchar tráfico                              │
│  └─ Volumen /data persistido entre deploys                             │
│     └─ Ref: ADR 0014                                                  │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  INF.V — SEGURIDAD VPS                                                 │
│  ├─ Usuario deploy (sin root)                                           │
│  ├─ SSH solo con clave pública (sin password)                          │
│  ├─ UFW: solo 22, 80, 443                                              │
│  └─ Unattended-upgrades para security updates                          │
│     └─ Ref: ADR 0014                                                  │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  INF.VI — MONITOREO Y ALERTAS                                          │
│  ├─ Healthchecks.io: Litestream, Apalis, TLS, Deploy                  │
│  ├─ Pings desde CleanupJob, Litestream, just deploy                    │
│  └─ Alertas: Email + Telegram                                          │
│     └─ Ref: ADR 0015, 0004, 0018                                      │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  INF.VII — CI/CD (GitHub Actions)                                     │
│  ├─ Triggers: push/PR a main                                           │
│  ├─ Steps: nextest, clippy, deny, audit, types-check, prepare          │
│  ├─ SQLX_OFFLINE=true (sin DB en CI)                                   │
│  └─ Build imagen distroless verificada                                 │
│     └─ Ref: ADR 0010, 0013                                            │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Documentación Oficial de Referencia

| Herramienta | URL | Útil para |
|----------------------|-----|-----------|
| **Kamal** | https://kamal-deploy.org | Zero-downtime deploy, rollback, healthchecks |
| **Caddy** | https://caddyserver.com/docs | Reverse proxy, TLS automático, headers |
| **Litestream** | https://litestream.io | Streaming backups SQLite a S3 |
| **Tigris S3** | https://www.tigrisdata.com/docs | S3-compatible storage |
| **Podman** | https://docs.podman.io | Containers rootless, daemonless |
| **Distroless** | https://github.com/GoogleContainerTools/distroless | Imágenes minimalistas sin shell |
| **Healthchecks.io** | https://healthchecks.io/docs | Monitoreo de cron jobs y servicios |
| **cargo-deny** | https://docs.rs/cargo-deny/latest | Licencias, CVEs, dependencias prohibidas |
| **cargo-audit** | https://docs.rs/cargo-audit/latest | Vulnerabilidades en dependencias |
| **UFW** | https://help.ubuntu.com/community/UFW | Firewall simple |
| **Let's Encrypt** | https://letsencrypt.org/docs | TLS gratis y automático |

---

## Troubleshooting — Infraestructura

### INF.I — Containerfile

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Build falla "sqlx query not found" | SQLX_OFFLINE=false o .sqlx/ no en git | `just prepare` y commitear — Ref: ADR 0013 |
| Imagen > 15MB | Usando Alpine/Ubuntu en lugar de distroless | Cambiar a `gcr.io/distroless/cc-debian12` — Ref: ADR 0013 |
| Container tiene shell | Imagen no es distroless | Verificar `FROM gcr.io/distroless/cc-debian12` — Ref: ADR 0013 |
| Litestream no encuentra config | Config no copiada en imagen | COPY `litestream.yml` a `/etc/litestream.yml` — Ref: ADR 0004 |
| Binary no ejecuta | Compilado para musl sin toolchain | Instalar `musl-tools` en builder — Ref: ADR 0013 |

### INF.II — Caddy

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| HTTPS no funciona | Puerto 443 no abierto en UFW | `ufw allow 443` — Ref: ADR 0014 |
| Headers de seguridad no aparecen | Caddyfile mal configurado | Revisar bloque `header {}` — Ref: ADR 0014 |
| Cache no funciona | `@static` no definido | Añadir `@static path /assets/* ...` — Ref: ADR 0014 |
| Logs no son JSON | Formato no configurado | `format json` en log block — Ref: ADR 0016 |
| Redirección HTTP→HTTPS no funciona | Caddy no puede escuchar 80 | Verificar `ufw allow 80` — Ref: ADR 0014 |

### INF.III — Litestream

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Backup no aparece en S3 | Credenciales AWS mal configuradas | Verificar `AWS_ACCESS_KEY_ID` — Ref: ADR 0020 |
| WAL no replica | sync-interval muy alto o endpoint mal | Revisar `litestream.yml` — Ref: docs/02-STACK.md L168 |
| Restore falla "bucket not found" | Bucket no existe o nombre mal | Verificar `LITESTREAM_BUCKET` — Ref: ADR 0004 |
| Integrity check falla | Backup corrupto o incompleto | Probar snapshot más antiguo — Ref: ADR 0004 |
| Litestream no arranca | `replicate` sin exec | Usar `litestream replicate -exec /api` — Ref: ADR 0004 |

### INF.IV — Kamal

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Deploy falla en healthcheck | Health endpoint no responde | Verificar `/health` en router — Ref: docs/03-STRUCTURE.md L278 |
| Rollback no funciona | Imagen anterior no existe | Verificar `kamal app images` — Ref: ADR 0014 |
| Zero-downtime no funciona | Healthcheck muy lento | Reducir `max_attempts` o aumentar `interval` — Ref: ADR 0014 |
| SSH connection refused | Clave no en VPS o usuario mal | Copiar `id_rsa.pub` a `~deploy/.ssh/authorized_keys` — Ref: ADR 0014 |
| Secret no disponible | No pasada en `env.secret` | Añadir a `deploy.yml` — Ref: ADR 0014 |

### INF.V — Seguridad VPS

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| SSH password rechazado | PasswordAuthentication no deshabilitado | Verificar `/etc/ssh/sshd_config` — Ref: ADR 0014 |
| Root login funciona | PermitRootLogin no | `PermitRootLogin no` — Ref: ADR 0014 |
| Puerto 8080 accesible desde afuera | UFW no configurado | `ufw deny 8080` — Ref: ADR 0014 |
| Usuario deploy no ve logs | No está en grupo systemd-journal | `usermod -aG systemd-journal deploy` — Ref: ADR 0014 |

### INF.VI — Monitoreo

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| Healthchecks.io no recibe ping | UUID mal configurado | Verificar `.env` con `HC_*_UUID` — Ref: ADR 0015 |
| Alerta de falso positivo | Grace period muy corto | Aumentar grace en check — Ref: ADR 0015 |
| Litestream check falla | Script de ping no ejecutado | Verificar script post-replicación — Ref: docs/02-STACK.md L167 |
| No llegan alertas a Telegram | Bot token/chat ID mal | Revisar configuración de notificaciones — Ref: ADR 0015 |

### INF.VII — CI/CD

| Síntoma | Causa probable | Solución |
|---------|---------------|----------|
| CI lento | Sin cache de cargo | Configurar `~/.cargo/registry` cache — Ref: docs/02-STACK.md L443 |
| cargo deny falla | jsonwebtoken en Cargo.toml | Eliminar dependencia — Ref: ADR 0008 |
| just prepare falla | SQLx queries desactualizadas | Correr localmente y commitear `.sqlx/` — Ref: ADR 0013 |
| types-check falla | api.ts no actualizado | `buf generate` y commitear — Ref: ADR 0027 |
| Build en CI funciona pero deploy no | Diferencia de arquitectura | Usar `x86_64-unknown-linux-musl` — Ref: ADR 0013 |

---

**Nota:** Si un error persiste, revisar los ADRs 0013 (Build), 0014 (Deploy), 0004 (Litestream), 0015 (Monitoreo) que son los más relevantes para infraestructura.
