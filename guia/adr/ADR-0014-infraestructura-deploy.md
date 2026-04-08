# ADR 0014 — Deploy: Podman + Caddy + Kamal

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0003 (binario estático Rust), ADR 0004 (Litestream sidecar), ADR 0012 (just deploy) |

---

## Contexto

El deploy debe cumplir con:

- **Aislamiento** de procesos sin la complejidad de Kubernetes
- **TLS automático** sin gestión manual de certificados
- **Zero-downtime** en cada deploy — el usuario no nota los despliegues
- **Rollback instantáneo** si algo falla
- **Funcionar en un VPS de $5** sin overhead de orquestadores

El objetivo es un sistema que un developer pueda deployar solo, desde su máquina, con un comando.

---

## Decisión

**Podman rootless** para contenedores, **Caddy** como reverse proxy con TLS automático,
**Kamal** como herramienta de deploy zero-downtime.

### Por qué Podman rootless en lugar de Docker

| Aspecto | Docker | Podman rootless |
|---------|--------|-----------------|
| Daemon | Requiere `dockerd` como root | Sin daemon — fork/exec directo |
| Socket | UNIX socket privilegiado | Sin socket privilegiado |
| Superficie de ataque | Daemon root expuesto | Mínima |
| Modo rootless | Disponible pero con fricciones | Modo por defecto |

### Containerfile — multi-stage con distroless

```dockerfile
# infra/docker/Containerfile

# ── Stage 1: Build ─────────────────────────────────────────────────────────────
FROM rust:1.82-slim AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y musl-tools pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Target estático — sin dependencias de libc en runtime
RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY crates/     ./crates/
COPY apps/api/   ./apps/api/

# SQLX_OFFLINE=true — el build no necesita acceso a la DB (ver ADR 0013)
ENV SQLX_OFFLINE=true
RUN cargo build --release \
    --target x86_64-unknown-linux-musl \
    --bin api

# ── Stage 2: Runtime mínimo ────────────────────────────────────────────────────
# distroless/cc: solo glibc y libssl — sin shell, sin package manager
FROM gcr.io/distroless/cc-debian12

COPY --from=builder \
    /app/target/x86_64-unknown-linux-musl/release/api \
    /api

COPY --from=builder /app/data/migrations /migrations

# Litestream como sidecar de replicación SQLite (ADR 0004)
COPY --from=ghcr.io/benbjohnson/litestream:latest-amd64 \
    /usr/local/bin/litestream /litestream

COPY infra/litestream/litestream.yml /etc/litestream.yml

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["/api", "health"]

EXPOSE 8080

ENTRYPOINT ["/litestream", "replicate", "-exec", "/api"]
```

Imagen final: ~10–15MB. Sin shell en producción.

### Caddyfile

```
# infra/caddy/Caddyfile

tudominio.com {
    reverse_proxy localhost:8080
    encode gzip zstd

    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
        X-Content-Type-Options    "nosniff"
        X-Frame-Options           "DENY"
        Referrer-Policy           "strict-origin-when-cross-origin"
        Permissions-Policy        "geolocation=(), microphone=(), camera=()"
        -Server                   # Ocultar versión de Caddy
    }

    # Cache agresiva para assets estáticos
    @static path /assets/* /images/* /fonts/*
    header @static Cache-Control "public, max-age=31536000, immutable"

    log { output file /var/log/caddy/access.log; format json }
}

app.tudominio.com {
    reverse_proxy localhost:4321  # Astro SSR
    encode gzip zstd

    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains"
        X-Content-Type-Options    "nosniff"
        X-Frame-Options           "DENY"
    }
}
```

### Kamal — configuración completa

```yaml
# infra/kamal/deploy.yml

service: boilerplate
image:   ghcr.io/tuuser/boilerplate

servers:
  web:
    hosts:
      - 123.456.789.0
    options:
      network: host

registry:
  server:   ghcr.io
  username: tuuser
  password:
    - KAMAL_REGISTRY_PASSWORD

env:
  clear:
    PORT:        "8080"
    RUST_LOG:    "info,sqlx=warn"
    ENVIRONMENT: "production"
  secret:
    - DATABASE_URL
    - PASETO_SECRET
    - RESEND_API_KEY
    - AWS_ACCESS_KEY_ID
    - AWS_SECRET_ACCESS_KEY
    - AWS_ENDPOINT_URL_S3
    - LITESTREAM_BUCKET
    - SENTRY_DSN

volumes:
  - "/data/boilerplate:/data"

healthcheck:
  path:         /health
  port:         8080
  max_attempts: 10
  interval:     3s

ssh:
  user:      deploy
  keys_only: true
```

### Flujo de deploy completo

```bash
# Primera vez — configura el VPS
kamal setup

# Deploy estándar (via just deploy):
# 1. just audit → cargo deny check + cargo audit
# 2. just test  → cargo nextest run
# 3. kamal deploy:
#    a. Build de la imagen en local
#    b. Push al registry (ghcr.io)
#    c. SSH al VPS → pull de la nueva imagen
#    d. Arrancar nuevo contenedor
#    e. Healthcheck hasta que /health responde 200
#    f. Redirigir tráfico al nuevo contenedor
#    g. Detener el contenedor anterior
#    → Zero-downtime total
# 4. Ping a Healthchecks.io (ADR 0015)

# Rollback si algo falla — ~5 segundos
kamal rollback

# Verificar zero-downtime durante deploy:
while true; do
    curl -s -o /dev/null -w "%{http_code}\n" https://tudominio.com/health
    sleep 0.5
done
# Todos los status deben ser 200, ninguno 502/503
```

### Restaurar la base de datos desde backup

```bash
# Si el VPS muere completamente — restaurar en servidor nuevo
litestream restore \
    -o /data/boilerplate.db \
    s3://tu-bucket/boilerplate/db

# Verificar integridad antes de arrancar
sqlite3 /data/boilerplate.db "PRAGMA integrity_check;"
# Debe retornar: ok

# Lanzar el sistema en el nuevo VPS
just deploy
```

### Checklist de seguridad del VPS

```bash
# Usuario de deploy sin privilegios root
adduser deploy
usermod -aG systemd-journal deploy

# SSH solo con clave pública — en /etc/ssh/sshd_config:
# PasswordAuthentication no
# PermitRootLogin no

# Firewall mínimo
ufw allow ssh
ufw allow 80
ufw allow 443
ufw enable

# Actualizaciones automáticas de seguridad
apt install unattended-upgrades
dpkg-reconfigure unattended-upgrades
```

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| Docker + Compose | Daemon root — mayor superficie de ataque |
| Nginx | Configuración de TLS manual, más verboso que Caddy |
| Fly.io / Railway | Vendor lock-in, costo más alto a escala |
| Kubernetes | Overkill para un solo VPS — overhead operativo desproporcionado |
| Ansible + scripts | Más frágil y difícil de mantener que Kamal |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la seguridad y la visibilidad del despliegue:

| Herramienta | Propósito en el Deploy |
| :--- | :--- |
| **`crowdsec`** | **Seguridad Adaptativa:** Protege contra ataques de fuerza bruta y escaneos bloqueando IPs maliciosas en tiempo real. |
| **`kamal-proxy`** | **Routing Nativo:** El proxy oficial de Kamal que facilita despliegues zero-downtime con menos configuración. |
| **`tailscale`** | **Red Privada:** Permite cerrar el puerto SSH al público y acceder al VPS solo de forma privada y segura. |
| **`healthchecks.io`** | **Monitoreo de Jobs:** Notificación inmediata si el despliegue o la replicación de Litestream fallan. |

---

## Consecuencias

### ✅ Positivas

- Imagen final ~10–15MB — descarga y arranque rápidos
- Sin shell en producción — superficie de ataque mínima con distroless
- TLS completamente automático con Caddy — sin renovaciones manuales
- Zero-downtime garantizado por el healthcheck de Kamal
- Rollback en ~5 segundos si el healthcheck falla

### ⚠️ Negativas / Trade-offs

- Sin shell en la imagen distroless — imposible hacer `exec` interactivo para debugging
  → Usar `kamal app exec --reuse` con imagen de debug en emergencias:
    ```bash
    kamal app exec --reuse --interactive "sh"
    # Solo funciona si se tiene una imagen de debug configurada en kamal
    ```
  → En la práctica: los logs JSON de tracing son suficientes para diagnosticar el 99% de los problemas
  → `kamal app exec` permite correr comandos específicos sin shell interactiva
- `SQLX_OFFLINE=true` requiere actualizar `.sqlx/` cuando cambian las queries
  → `just prepare` antes de commitear cambios de SQL — incluido en el pre-push hook de lefthook
  → Si se olvida: el CI falla con "query not found in offline cache" — error claro y accionable

### Decisiones derivadas

- El volumen `/data/boilerplate` se monta en el contenedor — persiste entre deploys
- Los secrets van como env secrets de Kamal — nunca en la imagen ni en git
- El endpoint `/health` verifica conexión a DB antes de responder 200
- `SQLX_OFFLINE=true` en el Containerfile — el build no necesita acceso a la DB
