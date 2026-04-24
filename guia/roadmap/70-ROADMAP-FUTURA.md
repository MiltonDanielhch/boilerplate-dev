# Roadmap — Futura: Escalamiento Avanzado Post-Fase 3

> **Objetivo:** Plan de supervivencia para cuando el proyecto tenga éxito masivo y necesite escalar más allá de Fase 3.
>
> **Stack:** Tecnologías empresariales para millones de usuarios.
>
> **ADRs:** F-001 a F-008 (todos en `adr/futura/`)
>
> **Filosofía:** Cada tecnología se activa SOLO cuando el problema específico existe con datos de producción reales. No escalar prematuramente.
>
> **Garantía:** La arquitectura hexagonal (ADR 0001) asegura que cada cambio sea quirúrgico — solo adaptadores, nunca dominio.

---
y 
## Estados

```
[ ] Pendiente   [~] En progreso   [x] Completado   [!] Bloqueado
🔮 Futura      ⚠️ Evaluación
```

---

## Mapa de Escalamiento Completo

```
FASE 1 (MVP) ──────► FASE 2 ──────► FASE 3 ──────► FUTURA (Este documento)
     │                  │              │                  │
     │                  │              │                  │
     ▼                  ▼              ▼                  ▼
VPS $5 SQLite     NATS Workers    KMP Mobile      Enterprise Scale
1 proceso         2 binarios      Nativo 120Hz    Millones de users
~500 users        ~50k users      ~500k users     ~5M+ users

FUTURA — Niveles de escalamiento (activar en orden):
┌─────────────────────────────────────────────────────────────────────┐
│ Nivel 6 — Observabilidad Enterprise                                 │
│ OTel Stack: Loki + Tempo + Grafana                                  │
│ └─ Activar: VPS $40+, >30 días retención logs, >3 developers        │
├─────────────────────────────────────────────────────────────────────┤
│ Nivel 7 — Caché Distribuida                                         │
│ Redis Cluster (L2) + Moka (L1)                                      │
│ └─ Activar: >5 nodos API, caché >100MB por nodo                     │
├─────────────────────────────────────────────────────────────────────┤
│ Nivel 8 — CDN Global                                                │
│ Cloudflare / CloudFront                                             │
│ └─ Activar: >50% tráfico global, latencia >200ms usuarios lejanos   │
├─────────────────────────────────────────────────────────────────────┤
│ Nivel 9 — Persistencia Enterprise                                   │
│ PostgreSQL con réplicas / SurrealDB multi-modelo                    │
│ └─ Activar: >100k users activos, joins complejos, >100 writes/s      │
├─────────────────────────────────────────────────────────────────────┤
│ Nivel 10 — Orquestación                                             │
│ Kubernetes (K3s o managed)                                        │
│ └─ Activar: >10 servicios, auto-scaling crítico, multi-region      │
├─────────────────────────────────────────────────────────────────────┤
│ Nivel 11 — Búsqueda Avanzada                                        │
│ Elasticsearch / OpenSearch / Meilisearch                            │
│ └─ Activar: >1M documentos, fuzzy search, facets, <500ms queries    │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Progreso de Futura

| Nivel | Nombre | ADR | Estado | Progreso |
|-------|--------|-----|--------|----------|
| 6 | Observabilidad OTel | F-004 | 🔮 | 0% |
| 7 | Redis Cluster | F-007 | 🔮 | 0% |
| 8 | CDN Global | F-006 | 🔮 | 0% |
| 9a | PostgreSQL | F-002 | 🔮 | 0% |
| 9b | SurrealDB | F-001 | 🔮 | 0% |
| 10 | Kubernetes | F-005 | 🔮 | 0% |
| 11 | Elasticsearch | F-008 | 🔮 | 0% |

---

## FT.1 — Observabilidad Enterprise (Nivel 6)

> **Referencia:** ADR F-004 (OTel Stack), ADR 0016 (actual), ADR 0031
>
> **Tecnología:** Loki (logs) + Tempo (traces) + Grafana (dashboards) + Prometheus (métricas)

```
[ ] Criterio de activación (todos deben cumplirse):
    [ ] VPS está en plan $40+/mes (4+ vCPU, 8+ GB RAM)
    [ ] Se necesitan >30 días de retención de logs históricos
    [ ] >3 developers necesitan acceso simultáneo a logs/traces
    [ ] Sentry ya no es suficiente para el volumen de errores
    [ ] Se necesita correlación log ↔ trace ↔ métrica en un clic
    └─ Ref: ADR F-004

[ ] Infraestructura:
    [ ] Crear infra/docker/compose.observability.yml:
        [ ] Grafana (dashboards unificados)
        [ ] Loki (logs con retención 30+ días)
        [ ] Tempo (traces distribuidos)
        [ ] Prometheus (métricas con alerting)
        [ ] Alertmanager (alerting avanzado)
    └─ Ref: ADR F-004

[ ] Configurar Rust:
    [ ] opentelemetry-otlp para exportar traces a Tempo
    [ ] tracing-loki o promtail sidecar para logs
    [ ] metrics crate para métricas personalizadas
    [ ] Mantener Sentry para errores críticos (complementario, no reemplazo)
    └─ Ref: ADR F-004

[ ] Dashboards:
    [ ] API Health: request rate, latency P95, error rate
    [ ] Logs: {service="boilerplate-api"} |= "ERROR"
    [ ] Traces: flame graphs por trace_id
    [ ] Infra: CPU, RAM, disco, network

[ ] Alerting:
    [ ] HighErrorRate: rate(http_5xx[5m]) > 0.1
    [ ] HighLatencyP95: histogram_quantile(0.95, http_duration) > 0.5s
    [ ] DatabaseConnectionErrors
    └─ Ref: ADR F-004
```

**Verificación FT.1:** Grafana muestra dashboards, alerts funcionan, logs retienen 30+ días.

---

## FT.2 — Caché Distribuida (Nivel 7)

> **Referencia:** ADR F-007 (Redis Cluster), ADR 0017 (Moka actual), ADR 0031
>
> **Tecnología:** Redis Cluster (3 masters + 3 replicas) + Moka local (L1)

```
[ ] Criterio de activación (cualquiera):
    [ ] >5 réplicas del API necesitan caché compartida
    [ ] Caché local >100MB por nodo (excede RAM)
    [ ] Necesidad de pub/sub entre nodos
    [ ] Rate limiting distribuido requerido
    [ ] Sesiones compartidas entre múltiples nodos
    └─ Ref: ADR F-007

[ ] Infraestructura:
    [ ] Crear infra/docker/compose.redis-cluster.yml:
        [ ] 3 Redis masters (sharding automático)
        [ ] 3 Redis replicas (HA)
        [ ] Redis Cluster init
    └─ Ref: ADR F-007

[ ] Implementar TieredCache en Rust:
    [ ] L1: Moka local (hot data, TTL <60s, ~1μs)
    [ ] L2: Redis Cluster (shared, TTL >60s, ~1ms)
    [ ] Fallback a DB si miss en ambos
    └─ Ref: ADR F-007

[ ] Features avanzadas:
    [ ] Rate limiting distribuido por IP/user
    [ ] Pub/sub para notificaciones en tiempo real
    [ ] Leaderboards con sorted sets
    [ ] Geospatial queries (si aplica)
    └─ Ref: ADR F-007
```

**Verificación FT.2:** Todos los nodos API ven la misma caché, latencia L2 <2ms.

---

## FT.3 — CDN Global (Nivel 8)

> **Referencia:** ADR F-006 (CDN), ADR 0014 (Caddy actual), ADR 0022 (Frontend)
>
> **Tecnología:** Cloudflare (preferido) o AWS CloudFront

```
[ ] Criterio de activación (todos deben cumplirse):
    [ ] >50% del tráfico proviene de fuera de la región del VPS
    [ ] Latencia >200ms TTFB para usuarios en continentes distantes
    [ ] Assets estáticos representan >50% del bandwidth
    [ ] Necesidad de DDoS protection y WAF
    └─ Ref: ADR F-006

[ ] Cloudflare (Free o Pro):
    [ ] Transferir NS a Cloudflare (o usar CNAME)
    [ ] Configurar Proxy status: Proxied (naranja)
    [ ] Page Rules:
        [ ] /assets/* → Cache Everything, TTL 24h
        [ ] /api/* → Bypass (nunca cachear API)
    [ ] SSL: Full (strict) con certificado origin
    [ ] Opcional: Cloudflare Pro ($20/mes) para WAF
    └─ Ref: ADR F-006

[ ] Configurar Caddy:
    [ ] Headers Cache-Control para assets vs API
    [ ] CF-Connecting-IP para logs (ver IP real del usuario)
    [ ] Rate limiting ajustado (Cloudflare ya filtra)
    └─ Ref: ADR F-006

[ ] Astro:
    [ ] Cache-busting con hash en nombres de archivo (automático)
    [ ] assetsPrefix si se usa dominio separado para CDN
    └─ Ref: ADR 0022
```

**Verificación FT.3:** WebPageTest muestra <100ms TTFB desde múltiples continentes.

---

## FT.4 — Persistencia Enterprise (Nivel 9a: PostgreSQL)

> **Referencia:** ADR F-002 (PostgreSQL), ADR 0004 (SQLite actual), ADR 0031
>
> **Tecnología:** PostgreSQL 16 con réplicas de lectura (cuando sea necesario)

```
[ ] Criterio de activación (cualquiera):
    [ ] >100k usuarios activos concurrentes
    [ ] >100 writes/segundo sostenidos
    [ ] Necesidad de joins analíticos complejos (OLAP)
    [ ] Herramientas BI que requieren PostgreSQL específicamente
    [ ] Necesidad de réplicas de lectura en múltiples regiones
    └─ Ref: ADR F-002

[ ] Migración quirúrgica (gracias a ADR 0001):
    [ ] Solo cambiar crates/database/src/pool.rs:
        [ ] De: SqlitePoolOptions → A: PgPoolOptions
    [ ] Adaptar queries:
        [ ] ? → $1, $2, etc.
        [ ] datetime('now') → NOW()
        [ ] TEXT UUID → UUID nativo
    [ ] Dominio y tests: SIN CAMBIOS
    └─ Ref: ADR F-002

[ ] Infraestructura:
    [ ] Crear infra/docker/compose.postgres.yml:
        [ ] PostgreSQL 16 Alpine
        [ ] Réplica de lectura (cuando sea necesaria)
        [ ] Backups con pg_basebackup o Barman
    [ ] Migración de datos:
        [ ] sqlite3 .dump → adaptar → psql import
        [ ] Verificar integridad: conteos de filas
    └─ Ref: ADR F-002

[ ] Litestream → pg_basebackup:
    [ ] Configurar backups continuos de PostgreSQL
    [ ] O usar Barman para backups WAL
    └─ Ref: ADR F-002, ADR 0004
```

**Verificación FT.4:** queries <50ms, replicación funciona, backups verificados.

---

## FT.5 — Persistencia Multi-Modelo (Nivel 9b: SurrealDB)

> **Referencia:** ADR F-001 (SurrealDB), ADR F-002 (PostgreSQL alternativa)
>
> **Tecnología:** SurrealDB con RocksDB backend (cuando PostgreSQL no alcance)

```
[ ] Criterio de activación (PostgreSQL ya no es suficiente):
    [ ] Consultas de grafo reales (traversal de >5 niveles de relaciones)
    [ ] Time-series: >1M eventos/día con agregaciones temporales
    [ ] Esquemas variables que no encajan en SQL relacional (multitenancy >1000 tenants)
    [ ] Necesidad de SQL + grafos + documentos + time-series en un solo motor
    └─ Ref: ADR F-001

[ ] Evaluación:
    [ ] Probar SurrealDB en staging con datos reales
    [ ] Comparar performance vs PostgreSQL para queries críticas
    [ ] Evaluar curva de aprendizaje del equipo (SurrealQL)
    └─ Ref: ADR F-001

[ ] Migración:
    [ ] Reescribir crates/database para SurrealDB
    [ ] Adaptar repositorios a SurrealQL
    [ ] Migración de datos desde PostgreSQL
    └─ Nota: Esta SÍ requiere cambios significativos — evaluar bien antes

[ ] Alternativa:
    [ ] Si solo se necesita una feature específica (ej: grafo), evalar Neo4j específicamente
    [ ] No cambiar toda la DB por una feature si se puede usar servicio separado
    └─ Ref: ADR F-001
```

**Nota:** SurrealDB es más riesgosa que PostgreSQL. Evaluar exhaustivamente antes.

---

## FT.6 — Orquestación (Nivel 10: Kubernetes)

> **Referencia:** ADR F-005 (Kubernetes), ADR 0014 (Kamal actual), ADR 0031
>
> **Tecnología:** K3s (on-premise) o managed K8s (DOKS, EKS, GKE)

```
[ ] Criterio de activación (todos deben cumplirse):
    [ ] >10 microservicios independientes
    [ ] Auto-scaling automático crítico (tráfico muy variable)
    [ ] Multi-region deploy obligatorio
    [ ] Equipo DevOps/SRE dedicado (>=2 personas)
    [ ] Costo de gestionar VPS manualmente supera K8s managed
    └─ Ref: ADR F-005

[ ] Evaluación previa:
    [ ] ¿Es realmente necesario K8s o Kamal + más VPS es suficiente?
    [ ] ¿El equipo tiene expertise en K8s?
    [ ] ¿El presupuesto justifica $100-500/mes en infraestructura?
    └─ Ref: ADR F-005

[ ] Opción A: K3s (mantener control de costos):
    [ ] 1 master + 2+ workers en VPS propios
    [ ] Carga de trabajo ligera (K3s es Kubernetes minimalista)
    [ ] Costo: solo VPS (sin costo de managed K8s)
    └─ Ref: ADR F-005

[ ] Opción B: Managed Kubernetes:
    [ ] DigitalOcean Kubernetes (DOKS) — más barato, bueno para startups
    [ ] AWS EKS — enterprise, servicios AWS integrados
    [ ] GKE — si se usa GCP para BigQuery/Dataflow
    └─ Ref: ADR F-005

[ ] Manifests K8s:
    [ ] Namespace: boilerplate
    [ ] Deployments: api, worker (con HPA para auto-scaling)
    [ ] StatefulSets: NATS, PostgreSQL (si no se usa managed)
    [ ] Services + Ingress (NGINX o Traefik)
    [ ] ConfigMaps + Secrets
    [ ] PersistentVolumeClaims para datos
    └─ Ref: ADR F-005

[ ] CI/CD:
    [ ] GitHub Actions con kubectl
    [ ] Helm charts para templating
    [ ] ArgoCD para GitOps (opcional)
    └─ Ref: ADR F-005
```

**Verificación FT.6:** kubectl get pods muestra todos running, HPA escala automáticamente, zero-downtime deploys.

---

## FT.7 — Búsqueda Avanzada (Nivel 11: Elasticsearch)

> **Referencia:** ADR F-008 (Elasticsearch), ADR 0004 (SQLite FTS5 actual)
>
> **Tecnología:** Elasticsearch, OpenSearch, o Meilisearch (según necesidad)

```
[ ] Criterio de activación:
    [ ] >1M documentos para indexar
    [ ] Necesidad de: fuzzy search, autocompletado, facets, highlight
    [ ] Queries de búsqueda >500ms en SQLite/PostgreSQL consistentemente
    [ ] Necesidad de: aggregations, geo-search, ML ranking
    └─ Ref: ADR F-008

[ ] Evaluación de alternativas:
    [ ] Meilisearch: Simple, rápido, Rust, self-hosted — para <10M docs
    [ ] Typesense: Open source, rápido, faceted search — para <10M docs  
    [ ] Elasticsearch: Full features, maduro — para >10M docs, ML ranking
    [ ] OpenSearch: Fork libre de ES — para evitar licencia Elastic
    └─ Ref: ADR F-008

[ ] Implementación:
    [ ] Elasticsearch Cluster (3 nodos: 1 master + 2 data)
    [ ] Index mapping con analyzers, sinónimos, completion suggest
    [ ] Indexer desde DB: CDC con Debezium o polling periódico
    [ ] Cliente Rust: elasticsearch crate
    └─ Ref: ADR F-008

[ ] Features:
    [ ] Fuzzy matching (typo tolerance)
    [ ] Autocompletado en tiempo real (completion suggester)
    [ ] Facets (filtros por categoría, tags, etc.)
    [ ] Highlight de términos buscados
    [ ] Aggregations (estadísticas, histograms)
    └─ Ref: ADR F-008
```

**Verificación FT.7:** Búsquedas <100ms, autocompletado <50ms, facets funcionan.

---

## Árbol de Decisión Completo (Post-Fase 3)

```
¿El sistema va lento después de Fase 3?
    ↓
¿El VPS tiene <8GB RAM?
    Sí → Subir a VPS $80+ (Nivel 1 extendido) — Ref: ADR 0031
    No ↓
¿Necesitas >30 días de logs y correlación trace/log?
    Sí → FT.1: OTel Stack (Loki + Tempo + Grafana) — Ref: ADR F-004
    No ↓
¿Tienes >5 nodos API necesitando caché compartida?
    Sí → FT.2: Redis Cluster — Ref: ADR F-007
    No ↓
¿>50% tráfico es global con latencia >200ms?
    Sí → FT.3: CDN Global (Cloudflare) — Ref: ADR F-006
    No ↓
¿Necesitas joins analíticos complejos o >100k users concurrentes?
    Sí → FT.4: PostgreSQL — Ref: ADR F-002
    No ↓
¿Necesitas grafos o time-series masivo?
    Sí → FT.5: SurrealDB — Ref: ADR F-001
    No ↓
¿Tienes >10 servicios y necesitas auto-scaling?
    Sí → FT.6: Kubernetes — Ref: ADR F-005
    No ↓
¿Tienes >1M documentos y necesitas búsqueda avanzada?
    Sí → FT.7: Elasticsearch — Ref: ADR F-008
    No → El sistema escala bien — disfruta el éxito
```

---

## Orden de Activación Recomendado

| Orden | Nivel | Tecnología | Trigger típico | Complejidad |
|-------|-------|------------|------------------|---------------|
| 1 | 6 | OTel Stack | VPS $40+, equipo creciendo | Media |
| 2 | 8 | CDN Global | Usuarios globales | Baja (Cloudflare Free) |
| 3 | 7 | Redis Cluster | Múltiples nodos API | Media |
| 4 | 9a | PostgreSQL | >100k users, writes pesadas | Media |
| 5 | 10 | Kubernetes | >10 servicios | Alta |
| 6 | 11 | Elasticsearch | >1M documentos | Media-Alta |
| 7 | 9b | SurrealDB | PostgreSQL no alcanza | Alta |

**Nota:** Este orden es una guía, no una regla estricta. Evaluar según el caso de uso específico.

---

## Checklist de "Éxito Masivo"

Cuando el proyecto tenga éxito, verificar cada nivel antes de activar:

```
Nivel 6 — Observabilidad:
[ ] Grafana accesible por todo el equipo
[ ] Dashboards: API health, logs, traces, infra
[ ] Alerts: HighErrorRate, HighLatency, DBConnectionErrors
[ ] Retención: 30+ días de logs

Nivel 7 — Redis:
[ ] Latencia L2 <2ms
[ ] Hit rate >80% en caché
[ ] Pub/sub funcionando entre nodos
[ ] Rate limiting distribuido activo

Nivel 8 — CDN:
[ ] TTFB <100ms desde 3+ continentes
[ ] Cache hit rate >90% en assets
[ ] DDoS protection activa
[ ] WAF configurado (si aplica)

Nivel 9 — PostgreSQL:
[ ] Queries <50ms P95
[ ] Réplica de lectura configurada (si aplica)
[ ] Backups automáticos funcionando
[ ] Migración desde SQLite verificada

Nivel 10 — Kubernetes:
[ ] kubectl get nodes: todos Ready
[ ] HPA escala automáticamente 3→20 pods
[ ] Rolling updates sin downtime
[ ] StatefulSets para NATS/DB operativos

Nivel 11 — Elasticsearch:
[ ] Búsquedas <100ms
[ ] Autocompletado <50ms
[ ] Facets funcionando
[ ] Indexación en tiempo real (refresh 1s)
```

---

## Costo Estimado por Nivel (mensual)

| Nivel | Tecnología | Infraestructura | Costo | Capacidad |
|-------|------------|-----------------|-------|-----------|
| 6 | OTel Stack | VPS $40 | $40 | Logs 30 días |
| 7 | Redis | VPS adicional $20 | $20 | Caché compartida |
| 8 | CDN | Cloudflare Pro | $20 | Global CDN |
| 9a | PostgreSQL | VPS $40 + managed | $60 | >100k users |
| 9b | SurrealDB | VPS $40 | $40 | Multi-modelo |
| 10 | Kubernetes | 3x VPS $20 + DOKS | $100 | Auto-scaling |
| 11 | Elasticsearch | 3x VPS $40 | $120 | >1M docs |
| **Total** | | | **~$400/mes** | **Millones de users** |

**Nota:** Estos son costos conservadores. Cloudflare Free = $0, K3s vs managed ahorra, etc.

---

## Documentación de Referencia

| Documento | Descripción |
|-----------|-------------|
| `ADR F-001` | SurrealDB multi-modelo |
| `ADR F-002` | PostgreSQL + réplicas |
| `ADR F-003` | KMP + UniFFI (ya en FASE3) |
| `ADR F-004` | OTel Stack (Loki + Tempo + Grafana) |
| `ADR F-005` | Kubernetes (K3s o managed) |
| `ADR F-006` | CDN Global (Cloudflare) |
| `ADR F-007` | Redis Cluster |
| `ADR F-008` | Elasticsearch / OpenSearch |
| `50-ROADMAP-FASE2.md` | NATS + Workers |
| `60-ROADMAP-FASE3.md` | KMP + Mobile Nativo |

---

## Troubleshooting — Futura

### Problema: Costo escala rápido

| Síntoma | Solución |
|---------|----------|
| $400/mes es mucho | Usar Cloudflare Free ($0), K3s en vez de managed ($60 ahorro), posponer ES |
| VPS se llena rápido | CDN para assets (FT.3 antes de FT.10) |
| Backups costosos | Litestream S3 es barato, pg_basebackup comprimido |

### Problema: Complejidad opera

| Síntoma | Solución |
|---------|----------|
| K8s muy complejo | Mantener Kamal + más VPS por más tiempo |
| ES muy pesado | Usar Meilisearch o Typesense (más ligeros) |
| Redis cluster caído | Fallback a Moka solamente (graceful degradation) |

### Problema: Migración dolorosa

| Síntoma | Prevención |
|---------|------------|
| PostgreSQL migration fail | Testear en staging con datos reales, mantener SQLite como fallback |
| K8s deploy falla | Mantener Kamal config como rollback, blue-green deploys |
| ES index corrupto | Reindex desde DB (source of truth), snapshots diarios |

---

## Notas Finales

1. **No escalar prematuramente:** Cada nivel tiene criterios concretos. Activar sin el problema real es deuda técnica.

2. **Arquitectura hexagonal:** ADR 0001 garantiza que cada cambio sea quirúrgico. El dominio nunca cambia.

3. **Medir antes de escalar:** Sin métricas de producción, no hay justificación para activar ningún nivel.

4. **Costo vs beneficio:** Cada nivel debe justificar su costo en tiempo de ingeniería + infraestructura.

5. **Rollback siempre:** Mantener la opción de volver al nivel anterior si algo falla.

6. **Documentar decisiones:** Si se salta un nivel o se activa temprano, documentar por qué en este roadmap.

---

**Última actualización:** 2026-04-04
**Próxima revisión:** Cuando se active cualquier nivel de Futura
**Responsable:** Staff Engineer + DevOps Lead
