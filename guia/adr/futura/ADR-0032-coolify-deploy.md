# ADR 0032 — Deploy alternativo: Coolify

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado — alternativa válida a Kamal |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0014 (Deploy Kamal — opción principal) · ADR 0004 (SQLite + Litestream) · ADR 0013 (Build Distroless) · ADR VPS (5 Pilares) |

---

## Contexto

El stack actual usa **Kamal** como herramienta de deploy (ADR 0014). Kamal opera desde
la laptop del desarrollador: compila localmente, sube la imagen al registry, y la despliega
en el VPS vía SSH. Es minimalista, sin dependencias extra en el servidor.

**Coolify** es una alternativa: un PaaS self-hosted de código abierto que se instala en
el VPS y provee un dashboard web para gestionar deploys, dominios, SSL, variables de
entorno y volúmenes persistentes. Funciona con Docker y Git.

La pregunta es: ¿puede Coolify desplegar este proyecto con SQLite + Litestream sin
comprometer la arquitectura?

---

## Análisis: Coolify + este stack

### ✅ Lo que funciona bien

**1. Containerfile existente — sin cambios**
Coolify despliega cualquier imagen Docker. El Containerfile multi-stage distroless
del ADR 0013 funciona tal cual. Coolify lo construye o consume la imagen pre-construida
desde el registry.

**2. SSL automático — mejor integrado que con Kamal**
Coolify usa Traefik internamente para el reverse proxy. Los certificados Let's Encrypt
se gestionan desde el dashboard con cero configuración manual. No se necesita Caddy
separado (aunque Caddy puede coexistir).

**3. Variables de entorno con interfaz visual**
Todas las variables del `.env` (PASETO_SECRET, DATABASE_URL, RESEND_API_KEY, etc.)
se configuran desde el dashboard con un editor visual. Los secrets se cifran en reposo.

**4. Despliegues desde Git**
Push a `main` → Coolify detecta el cambio vía webhook → construye la imagen →
despliega automáticamente. Elimina el paso manual de `just deploy`.

**5. Múltiples apps en el mismo VPS**
Coolify gestiona varias aplicaciones desde un solo dashboard. Útil cuando el VPS aloja
tanto la API Axum como el frontend Astro SSR.

**6. Healthcheck integrado**
El endpoint `/health` del ADR 0003 se configura en el dashboard como healthcheck.
Coolify no hace el swap al nuevo contenedor hasta que el healthcheck pasa.

---

### ⚠️ Consideraciones críticas para SQLite + Litestream

Esta es la parte que requiere atención. El stack usa SQLite WAL con Litestream como
sidecar (ADR 0004). Coolify gestiona contenedores Docker — el patrón sidecar no es
nativo en su modelo de deployment estándar.

**Problema:** Litestream corre como proceso junto al binario Rust dentro del mismo
contenedor (via `ENTRYPOINT ["/litestream", "replicate", "-exec", "/api"]`). Este
patrón funciona perfectamente con Kamal porque el contenedor es completamente autónomo.

**Con Coolify:** el contenedor funciona igual — Coolify solo gestiona cuándo se inicia
y para. El `ENTRYPOINT` de Litestream sigue funcionando. Lo que cambia es el **volumen
persistente** donde vive la DB.

---

## Decisión

Coolify es una alternativa válida a Kamal con **dos ajustes obligatorios** en la
configuración del deploy.

### Ajuste 1 — Volumen persistente para SQLite

En Coolify, el volumen se configura en el dashboard → "Persistent Storage":

```
Source Path:      (vacío — Coolify crea un volumen Docker nombrado)
Destination Path: /data
```

Esto monta `/data` en el contenedor como volumen persistente que sobrevive los redeploys.
La variable de entorno cambia a:

```env
DATABASE_URL=sqlite:/data/boilerplate.db
```

El Containerfile NO necesita cambios — Litestream sigue siendo el sidecar en el
mismo `ENTRYPOINT`.

### Ajuste 2 — Containerfile sin distroless para Coolify Cloud (solo si aplica)

Si se usa **Coolify Cloud** (el servicio gestionado, no self-hosted), puede haber
restricciones con imágenes distroless. En self-hosted no hay ningún problema.

Para self-hosted (el caso de este proyecto): el Containerfile distroless funciona
sin cambios.

---

## Cómo desplegar con Coolify

### Paso 1 — Instalar Coolify en el VPS

```bash
# En el VPS, después del Protocolo de los 5 Pilares (ADR VPS)
curl -fsSL https://cdn.coollabs.io/coolify/install.sh | sudo bash

# Coolify queda disponible en http://IP_DEL_VPS:8000
# Configurar dominio propio: coolify.tudominio.com → puerto 8000
```

**Requisitos de RAM:** Coolify consume ~500MB-1GB de RAM. En un VPS de $5 (1GB RAM)
con el Swap del ADR VPS activo, es ajustado pero funcional. Recomendado: VPS de $10
(2GB RAM) cuando se use Coolify.

### Paso 2 — Conectar el repositorio Git

En el dashboard de Coolify:

```
New Resource → Application → GitHub / GitLab / Gitea
→ Seleccionar el repositorio boilerplate
→ Branch: main
→ Build Pack: Dockerfile
→ Dockerfile location: infra/docker/Containerfile
```

### Paso 3 — Configurar variables de entorno

En el dashboard → "Environment Variables":

```env
SERVER_PORT=8080
ENVIRONMENT=production
RUST_LOG=info,sqlx=warn
DATABASE_URL=sqlite:/data/boilerplate.db
PASETO_SECRET=<32 bytes — generar con: openssl rand -hex 16>
RESEND_API_KEY=re_xxxx
MAIL_FROM=noreply@tudominio.com
AWS_ENDPOINT_URL_S3=https://fly.storage.tigris.dev
AWS_ACCESS_KEY_ID=tid_xxxx
AWS_SECRET_ACCESS_KEY=tsec_xxxx
STORAGE_BUCKET=boilerplate-production-assets
LITESTREAM_BUCKET=boilerplate-production-backups
SENTRY_DSN=https://xxx@sentry.io/xxx
HC_LITESTREAM_UUID=xxxx
HC_DEPLOY_UUID=xxxx
```

Marcar como **Secret** los valores sensibles: PASETO_SECRET, RESEND_API_KEY,
AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY.

### Paso 4 — Configurar volumen persistente

En el dashboard → "Persistent Storage" → "Add Volume":

```
Name:             boilerplate-data
Source Path:      (vacío)
Destination Path: /data
```

### Paso 5 — Configurar healthcheck y dominio

```
Healthcheck:
  Path: /health
  Port: 8080
  Interval: 10s
  Timeout: 5s
  Retries: 3

Dominio: tudominio.com → puerto 8080
```

### Paso 6 — Deploy inicial

```
Dashboard → Deploy → Build & Deploy
```

Coolify construye la imagen, ejecuta el contenedor con el volumen montado en `/data`,
Litestream inicia la replicación a S3, y las migraciones SQLx corren automáticamente
al arrancar (como siempre — esto no cambia).

---

## Comparación: Coolify vs Kamal para este stack

| Aspecto | Kamal (ADR 0014) | Coolify |
|---------|-----------------|---------|
| **Interfaz** | CLI desde la laptop | Dashboard web visual |
| **SSL** | Caddy (configura el desarrollador) | Traefik automático (Coolify lo gestiona) |
| **Deploy** | `just deploy` desde la laptop | Push a Git → webhook → deploy automático |
| **Variables de entorno** | Secrets de Kamal (`.kamal/secrets`) | Dashboard visual con cifrado en reposo |
| **Volúmenes persistentes** | Configurado en `deploy.yml` | Dashboard visual |
| **Rollback** | `kamal rollback` (~5s) | Dashboard → "Rollback" con historial visual |
| **Múltiples apps** | Un `deploy.yml` por app | Una sola instalación para todas las apps |
| **RAM extra requerida** | ~0MB (Kamal corre en la laptop) | ~500MB-1GB (Coolify corre en el VPS) |
| **SQLite + Litestream** | ✅ Nativo — sidecar en ENTRYPOINT | ✅ Funciona — volumen `/data` en dashboard |
| **Distroless** | ✅ Nativo | ✅ Funciona en self-hosted |
| **Curva de aprendizaje** | CLI (conocido si sabes SSH) | Dashboard web (más visual, más fácil) |
| **Dependencias en el VPS** | Solo Docker/Podman | Docker + Coolify (~10 contenedores) |
| **Costo extra** | $0 | $0 (open source) |
| **VPS mínimo** | $5 (1 vCPU / 1GB RAM) | $10 (1 vCPU / 2GB RAM) recomendado |

---

## Cuándo elegir Coolify sobre Kamal

**Elegir Coolify si:**
- Se gestionan múltiples proyectos en el mismo VPS y se quiere un dashboard unificado
- Se prefiere una interfaz visual sobre CLI
- El equipo tiene miembros que no están cómodos con SSH y comandos de terminal
- Se quiere auto-deploy en cada push a Git sin configurar GitHub Actions
- El VPS tiene ≥2GB de RAM disponible

**Mantener Kamal si:**
- VPS de $5 con 1GB de RAM — Coolify consume demasiada memoria en ese caso
- Se prefiere el control total desde la CLI sin dependencias extra en el servidor
- El equipo está cómodo con el flujo `just deploy`
- Se quiere la imagen más mínima posible (Coolify añade sus propios contenedores)

---

## Litestream en Coolify — detalles técnicos

El patrón de Litestream como sidecar dentro del mismo contenedor funciona igual en
Coolify. El flujo al iniciar el contenedor:

```
1. Coolify inicia el contenedor con el volumen /data montado
2. ENTRYPOINT: /litestream replicate -exec /api
3. Litestream verifica si existe /data/boilerplate.db
   → Si NO existe: restaura desde S3 antes de iniciar /api
   → Si existe: inicia replicación continua
4. /api arranca, ejecuta migraciones SQLx, empieza a servir tráfico
5. Healthcheck /health pasa → Coolify marca el deploy como exitoso
```

```yaml
# infra/litestream/litestream.yml — sin cambios
dbs:
  - path: /data/boilerplate.db
    replicas:
      - type: s3
        bucket: ${LITESTREAM_BUCKET}
        path: boilerplate/db
        endpoint: ${AWS_ENDPOINT_URL_S3}
        sync-interval: 1s
        snapshot-interval: 24h
        retention: 72h
```

---

## Containerfile — ajuste mínimo para Coolify

El Containerfile del ADR 0013 funciona sin cambios en Coolify self-hosted. El único
ajuste es que el directorio de datos debe ser `/data` (no hardcoded en el binary):

```dockerfile
# infra/docker/Containerfile — sin cambios respecto al ADR 0013
FROM rust:1.82-slim AS builder
RUN rustup target add x86_64-unknown-linux-musl
# ... build igual que siempre ...

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /target/x86_64-unknown-linux-musl/release/api /api
COPY --from=ghcr.io/benbjohnson/litestream:latest-amd64 \
     /usr/local/bin/litestream /litestream
COPY infra/litestream/litestream.yml /etc/litestream.yml
EXPOSE 8080
ENTRYPOINT ["/litestream", "replicate", "-exec", "/api"]
```

La variable `DATABASE_URL=sqlite:/data/boilerplate.db` en Coolify apunta al volumen
persistente montado en `/data`. Eso es todo.

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| **Dokploy** | Más nuevo y menos maduro que Coolify — ecosistema más pequeño |
| **CapRover** | Interfaz más anticuada, menos activo en 2026 |
| **Heroku** | Vendor lock-in, precio elevado, no self-hosted |
| **Fly.io** | Excelente para este stack pero no self-hosted — costos variables |
| **Railway** | No self-hosted — vendor lock-in |
| **Coolify Cloud** | Servicio gestionado de Coolify — evita autogestión pero añade costo mensual |

---

## Consecuencias

### ✅ Positivas

- Dashboard visual para gestionar deploys, logs, dominios y variables de entorno
- Auto-deploy en cada push a Git sin configurar CI/CD manualmente
- SSL automático con Traefik — cero configuración manual de certificados
- Rollback visual con historial de deploys
- El Containerfile, Litestream y la arquitectura hexagonal no cambian absolutamente nada
- Gestión de múltiples proyectos desde un solo panel

### ⚠️ Negativas / Trade-offs

- **Consume RAM extra (~500MB-1GB):** Coolify corre ~10 contenedores propios en el VPS
  → Mitigación: usar VPS de $10 (2GB RAM) en lugar del de $5. El costo sube pero
  el beneficio de gestionar múltiples proyectos desde un dashboard lo justifica
- **Complejidad añadida:** más contenedores corriendo = más superficie de fallo potencial
  → Mitigación: Coolify es maduro (48k+ estrellas en GitHub), tiene actualizaciones automáticas
- **No es la opción más minimalista:** para un solo proyecto en un VPS de $5, Kamal
  sigue siendo más eficiente en recursos
  → Coolify brilla cuando hay 2+ proyectos en el mismo servidor

### Decisiones derivadas

- `DATABASE_URL` usa `/data/boilerplate.db` (volumen de Coolify) en lugar de `./data/boilerplate.db`
- El resto de la arquitectura, el Containerfile, Litestream y los ADRs existentes no cambian
- El `justfile` mantiene `just deploy` como opción CLI — Coolify es una capa adicional, no un reemplazo que bloquea
- Si el VPS es de $5 con 1GB RAM → usar Kamal (ADR 0014). Si es de $10+ → Coolify es válido
