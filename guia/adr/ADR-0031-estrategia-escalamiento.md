# ADR 0031 — Escalamiento: 5 Niveles con Criterios

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0001 (Arquitectura Hexagonal), ADR 0004 (SQLite), ADR 0025 (NATS), ADR 0014 (Deploy), ADR 0030 (Multiplataforma) |

---

## Contexto

El boilerplate arranca en un VPS de $5 con SQLite y un proceso único. Si el proyecto crece,
el sistema debe poder escalar sin reescribir el núcleo.

La arquitectura hexagonal del ADR 0001 ya está diseñada para esto: solo cambian los
adaptadores, nunca el dominio.

**Regla de oro:** no escalar prematuramente. Cada nivel tiene un criterio concreto de
cuándo activarlo. Escalar antes de ese umbral es deuda técnica disfrazada de arquitectura.

---

## Los 5 niveles de escalamiento

### Nivel 1 — Escalamiento vertical (el más barato en tiempo)

**Cuándo activar:** antes de tocar una sola línea de código.
**Criterio concreto:** CPU >80% sostenido por >10 minutos, o RAM >85% sostenida.

```
VPS $5   (1 vCPU / 1GB RAM)   → MVP, hasta ~500 usuarios activos simultáneos
VPS $20  (2 vCPU / 4GB RAM)   → crecimiento inicial
VPS $40  (4 vCPU / 8GB RAM)   → escala media
```

**Qué cambia en el código:** nada. Rust + SQLite WAL + Moka aprovechan recursos adicionales automáticamente.

Verificar antes de subir de plan:

```bash
# 1. Optimizaciones de release activas (ADR 0013)
grep -A 5 "\[profile.release\]" Cargo.toml
# opt-level = "z", lto = true, strip = true

# 2. SQLite WAL configurado (ADR 0004)
# journal_mode=WAL, cache_size=-64000, mmap_size=30000000000

# 3. Moka limitado a 100MB (ADR 0017)
# Cache::builder().max_capacity(10_000)
```

---

### Nivel 2 — Optimización de persistencia

**Cuándo activar:** SQLite WAL llega a sus límites reales.
**Criterio concreto:** latencia P99 >100ms en `/health` durante >30 minutos sostenidos,
o >100 escrituras por segundo sostenidas.

```
SQLite WAL local (MVP)
    ↓ criterio: >100 writes/s sostenidos
Turso (SQLite distribuido)
    → misma API SQLx — solo cambiar DATABASE_URL
    → los repositorios NO se tocan
    ↓ criterio: joins analíticos complejos o réplicas de lectura
PostgreSQL
    → solo cambia crates/database/src/pool.rs
    → el dominio, casos de uso y tests: intactos
```

```rust
// crates/database/src/pool.rs — el ÚNICO archivo que cambia para PostgreSQL
// De:
SqlitePoolOptions::new().connect(&database_url).await

// A:
PgPoolOptions::new().connect(&database_url).await

// El resto del sistema: no cambia
```

---

### Nivel 3 — Almacenamiento de objetos

**Cuándo activar:** el disco del VPS empieza a llenarse con archivos binarios.
**Criterio concreto:** esta regla ya está activa desde el día uno.

```bash
# Regla inamovible: ningún archivo binario en el disco del VPS
# Fotos, documentos, PDFs → S3/Tigris desde el primer upload (ADR 0020)

# Plan B si Tigris no está disponible — solo cambiar el endpoint:
AWS_ENDPOINT_URL_S3=https://xxxx.r2.cloudflarestorage.com  # Cloudflare R2
```

---

### Nivel 4 — Desacoplamiento de servicios

**Cuándo activar:** los jobs pesados degradan la latencia HTTP.
**Criterio concreto:** latencia P95 del API >50ms cuando Apalis está procesando jobs pesados.

```
Antes (Fase 1 — monolito):
  apps/api/ → Axum HTTP + Apalis workers en el mismo proceso

Después (Fase 2 — dos binarios):
  apps/api/    → Solo Axum HTTP — latencia garantizada
  apps/worker/ → Solo Apalis workers — sin competir por CPU
```

```toml
# apps/worker/Cargo.toml — nuevo binario que reutiliza los mismos crates
[dependencies]
application = { path = "../../crates/application" }
database    = { path = "../../crates/database" }
mailer      = { path = "../../crates/mailer" }
# Los crates de dominio NO cambian
```

Comunicación vía NATS JetStream (ADR 0025):

```
apps/api (recibe request HTTP)
    → publica boilerplate.report.requested.v1
    → responde 202 Accepted inmediatamente

apps/worker (proceso separado)
    → consume boilerplate.report.requested.v1
    → genera el reporte PDF
    → notifica al usuario cuando termina
```

---

### Nivel 5 — Mobile nativo

**Cuándo activar:** la app móvil necesita rendimiento de clase mundial.
**Criterio concreto:** listas de >10.000 elementos lentas en Tauri, animaciones <60Hz constantes,
o procesamiento offline masivo en el dispositivo.

Ver ADR 0030 Fase 3 — KMP + UniFFI. Los crates de dominio se mantienen intactos.

---

## El árbol de decisión

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
¿Hay archivos binarios en el disco del VPS?
    Sí → moverlos a S3/Tigris inmediatamente (Nivel 3 / ADR 0020)
    No ↓
¿Los jobs pesados degradan la latencia HTTP?
    Sí → separar worker con NATS (Nivel 4 / ADR 0025)
    No ↓
¿La app móvil necesita 120Hz nativos?
    Sí → KMP + UniFFI (Nivel 5 / ADR 0030)
    No → el sistema escala bien para el caso de uso actual
```

---

## Capacidad estimada por nivel

| Nivel | Infraestructura | Usuarios activos | Costo/mes |
|-------|----------------|-----------------|-----------|
| 0 (MVP) | VPS $5, SQLite, 1 proceso | ~500 | $5 |
| 1 (vertical) | VPS $20, SQLite, 1 proceso | ~5.000 | $20 |
| 2a (Turso) | VPS $20, Turso | ~20.000 | $20-40 |
| 2b (Postgres) | VPS $40 + Postgres | ~100.000 | $60-80 |
| 4 (worker) | 2x VPS + NATS | ~100.000+ | $80-120 |

Rust es tan eficiente que estas estimaciones son conservadoras — Axum + SQLite WAL
maneja >10.000 requests/segundo en un VPS de $5 en benchmarks reales.

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| Kubernetes desde el inicio | Overkill — overhead operativo desproporcionado |
| Redis para caché desde el inicio | Proceso externo innecesario — Moka cubre el 90% en MVP |
| Microservicios desde el inicio | Cada microservicio es un VPS extra, CI extra, deploy extra |
| Escalar la DB antes del VPS | El VPS más grande es 10x más barato en tiempo de ingeniero |

---
## Herramientas y Librerías para Optimizar (Edición 2026)

Para medir y ejecutar el escalamiento con precisión quirúrgica:

| Herramienta | Propósito en el Escalamiento |
| :--- | :--- |
| **`netdata`** | **Monitoreo Real-time:** Dashboard ultra-ligero para detectar cuándo se alcanzan los límites de CPU/RAM del Nivel 1. |
| **`k6`** | **Load Testing:** Permite simular tráfico para validar si el sistema actual soporta el siguiente umbral de usuarios. |
| **`infracost`** | **Control de Costos:** Calcula el impacto financiero de subir de nivel antes de ejecutar el deploy. |
| **`pg-wal-g`** | **Postgres Backup:** Si se llega al Nivel 2b, es el equivalente a Litestream para mantener la seguridad de los datos. |

---
## Consecuencias

### ✅ Positivas

- El orden de escalamiento está documentado — no hay que decidir bajo presión
- Cada nivel tiene criterio claro de activación — sin escalar especulativamente
- La arquitectura hexagonal del ADR 0001 garantiza que los niveles 2-5 son quirúrgicos
- De MVP a 100k usuarios sin reescribir el dominio ni los tests

### ⚠️ Negativas / Trade-offs

- El Nivel 4 (separar worker) requiere configurar NATS en producción
  → La infraestructura de NATS está documentada en ADR 0026 — solo activar el compose
  → Los workers reutilizan exactamente los mismos crates — sin duplicación de código
- El Nivel 5 (KMP) requiere conocer Kotlin y configurar Android Studio + Xcode
  → Solo se activa con criterio concreto medido en producción real — no especulativamente
- Turso (Nivel 2a) introduce dependencia de servicio externo
  → La misma API SQLx garantiza que el código no cambia — solo la cadena de conexión

### Decisiones derivadas

- Ningún archivo binario en el disco del VPS desde el día uno — S3/Tigris siempre
- `[profile.release]` con `opt-level="z"` y `lto=true` activo desde el primer deploy
- El Nivel 1 (subir de plan) siempre se evalúa antes del Nivel 2 (cambiar DB)
- Si se activa el Nivel 4, `apps/worker/` reutiliza exactamente los mismos crates — sin duplicación
