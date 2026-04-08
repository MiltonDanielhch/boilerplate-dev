# ADR F-007 — Futuro: Redis Cluster (Caché Distribuida)

| Campo | Valor |
|-------|-------|
| **Estado** | 🔮 Futuro — activar cuando Moka ya no sea suficiente |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0017 (Moka Cache), ADR 0031 (Escalamiento Nivel 4+), ADR 0001 (Hexagonal) |

---

## Contexto

Este ADR documenta la migración a **Redis Cluster** cuando **Moka** (caché en-memory local)
ya no sea suficiente para la escala operativa. Redis proporciona caché distribuida compartida
entre múltiples instancias del API.

**Activar cuando:**
- Múltiples réplicas del API (K8s o múltiples VPS) necesitan caché compartida
- La caché local excede la RAM disponible en un solo nodo (>100MB)
- Necesidad de persistencia de caché entre reinicios
- Features avanzadas: pub/sub, rate limiting distribuido, sesiones compartidas

**NO activar hasta Fase 4+.** Moka es 10x más simple y suficiente hasta ~10 instancias del API.

---

## Decisión futura

Usar **Redis Cluster** (o Redis Sentinel) como caché distribuida compartida.

### Arquitectura

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Redis Cluster (3 masters, 3 replicas)          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                           │
│  │ Master 1 │──│ Master 2 │──│ Master 3 │ (sharding automático)    │
│  │ :6379    │  │ :6380    │  │ :6381    │                           │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘                           │
│       │             │             │                                 │
│  ┌────┴─────┐  ┌────┴─────┐  ┌────┴─────┐                           │
│  │ Replica 1│  │ Replica 2│  │ Replica 3│                           │
│  └──────────┘  └──────────┘  └──────────┘                           │
└───────────────────────────────┬───────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│   API Pod 1  │      │   API Pod 2  │      │   API Pod N  │
│  (Axum)      │      │  (Axum)      │      │  (Axum)      │
│              │      │              │      │              │
│ ┌──────────┐ │      │ ┌──────────┐ │      │ ┌──────────┐ │
│ │ Moka     │ │      │ │ Moka     │ │      │ │ Moka     │ │
│ │ (L1)     │ │      │ │ (L1)     │ │      │ │ (L1)     │ │
│ │ 10MB     │ │      │ │ 10MB     │ │      │ │ 10MB     │ │
│ └────┬─────┘ │      │ └────┬─────┘ │      │ └────┬─────┘ │
│      │       │      │      │       │      │      │       │
│      ▼       │      │      ▼       │      │      ▼       │
│  Redis (L2)  │      │  Redis (L2)  │      │  Redis (L2)  │
│  (compartido)│      │  (compartido)│      │  (compartido)│
└──────────────┘      └──────────────┘      └──────────────┘
```

### Jerarquía de caché (L1 + L2)

| Nivel | Tecnología | Scope | Latencia | Uso |
|-------|-----------|-------|----------|-----|
| L1 | Moka | Local al pod | ~1μs | Hot data, TTL <60s |
| L2 | Redis | Global | ~1ms | Shared cache, TTL >60s |
| DB | SQLite/PostgreSQL | Persistente | ~10ms | Source of truth |

---

## Cuándo activar

| Criterio | Umbral |
|----------|--------|
| Instancias API | >5 réplicas del API necesitan caché compartida |
| RAM caché | >100MB de datos cacheados (excede RAM por nodo) |
| Persistencia | Necesidad de sobrevivir reinicios con caché intacta |
| Features | Pub/sub, rate limiting distribuido, leaderboards, geospatial |
| Presupuesto | $50+/mes para infraestructura Redis justificable |

---

## Implementación

### 1. Redis Cluster (Docker Compose local)

```yaml
# infra/docker/compose.redis-cluster.yml
version: '3.8'
services:
  redis-master-1:
    image: redis:7.2-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-master-1-data:/data
    command: >
      redis-server
      --port 6379
      --cluster-enabled yes
      --cluster-config-file nodes.conf
      --cluster-node-timeout 5000
      --appendonly yes

  redis-master-2:
    image: redis:7.2-alpine
    ports:
      - "6380:6379"
    volumes:
      - redis-master-2-data:/data
    command: >
      redis-server
      --port 6379
      --cluster-enabled yes
      --cluster-config-file nodes.conf
      --cluster-node-timeout 5000
      --appendonly yes

  redis-master-3:
    image: redis:7.2-alpine
    ports:
      - "6381:6379"
    volumes:
      - redis-master-3-data:/data
    command: >
      redis-server
      --port 6379
      --cluster-enabled yes
      --cluster-config-file nodes.conf
      --cluster-node-timeout 5000
      --appendonly yes

  redis-replica-1:
    image: redis:7.2-alpine
    volumes:
      - redis-replica-1-data:/data
    command: >
      redis-server
      --port 6379
      --cluster-enabled yes
      --cluster-config-file nodes.conf
      --cluster-node-timeout 5000
      --appendonly yes
      --replicaof redis-master-1 6379

  redis-replica-2:
    image: redis:7.2-alpine
    volumes:
      - redis-replica-2-data:/data
    command: >
      redis-server
      --port 6379
      --cluster-enabled yes
      --cluster-config-file nodes.conf
      --cluster-node-timeout 5000
      --appendonly yes
      --replicaof redis-master-2 6379

  redis-replica-3:
    image: redis:7.2-alpine
    volumes:
      - redis-replica-3-data:/data
    command: >
      redis-server
      --port 6379
      --cluster-enabled yes
      --cluster-config-file nodes.conf
      --cluster-node-timeout 5000
      --appendonly yes
      --replicaof redis-master-3 6379

  # Inicializar cluster
  redis-cluster-init:
    image: redis:7.2-alpine
    depends_on:
      - redis-master-1
      - redis-master-2
      - redis-master-3
      - redis-replica-1
      - redis-replica-2
      - redis-replica-3
    command: >
      sh -c "
        sleep 5 &&
        redis-cli --cluster create
          redis-master-1:6379
          redis-master-2:6379
          redis-master-3:6379
          redis-replica-1:6379
          redis-replica-2:6379
          redis-replica-3:6379
          --cluster-replicas 1
          --cluster-yes
      "

volumes:
  redis-master-1-data:
  redis-master-2-data:
  redis-master-3-data:
  redis-replica-1-data:
  redis-replica-2-data:
  redis-replica-3-data:
```

### 2. Cliente Redis en Rust

```toml
# Cargo.toml
deadpool-redis = "0.14"
redis = { version = "0.24", features = ["tokio-comp", "cluster-async"] }
```

```rust
// crates/cache/src/redis_layer.rs
use deadpool_redis::{Config, Runtime};
use redis::AsyncCommands;

pub struct RedisCache {
    pool: deadpool_redis::Pool,
}

impl RedisCache {
    pub async fn new(redis_urls: Vec<String>) -> Result<Self, CacheError> {
        let cfg = Config::from_urls(redis_urls);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        
        Ok(Self { pool })
    }
    
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError> {
        let mut conn = self.pool.get().await?;
        let data: Option<Vec<u8>> = conn.get(key).await?;
        
        match data {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
    
    pub async fn set<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: u64,
    ) -> Result<(), CacheError> {
        let mut conn = self.pool.get().await?;
        let bytes = serde_json::to_vec(value)?;
        
        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl_seconds)
            .arg(bytes)
            .query_async::<_, ()>(&mut conn)
            .await?;
        
        Ok(())
    }
    
    pub async fn invalidate(&self, pattern: &str) -> Result<u64, CacheError> {
        let mut conn = self.pool.get().await?;
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await?;
        
        if !keys.is_empty() {
            redis::cmd("DEL")
                .arg(&keys)
                .query_async::<_, usize>(&mut conn)
                .await?;
        }
        
        Ok(keys.len() as u64)
    }
}
```

### 3. Integración con Moka (L1 + L2)

```rust
// crates/cache/src/tiered_cache.rs
use moka::future::Cache as MokaCache;

pub struct TieredCache<T: Clone + Send + Sync> {
    l1: MokaCache<String, T>,  // Local in-memory
    l2: Option<RedisCache>,   // Distributed Redis (optional)
}

impl<T: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned> TieredCache<T> {
    pub fn new(l1_capacity: u64, l2: Option<RedisCache>) -> Self {
        let l1 = MokaCache::new(l1_capacity);
        Self { l1, l2 }
    }
    
    pub async fn get(&self, key: &str) -> Result<Option<T>, CacheError> {
        // 1. Intentar L1 (Moka) - ~1μs
        if let Some(value) = self.l1.get(key).await {
            return Ok(Some(value));
        }
        
        // 2. Intentar L2 (Redis) - ~1ms
        if let Some(ref redis) = self.l2 {
            if let Some(value) = redis.get(key).await? {
                // Backfill L1
                self.l1.insert(key.to_string(), value.clone()).await;
                return Ok(Some(value));
            }
        }
        
        Ok(None)
    }
    
    pub async fn set(
        &self,
        key: &str,
        value: T,
        l1_ttl: std::time::Duration,
        l2_ttl: std::time::Duration,
    ) -> Result<(), CacheError> {
        // 1. Guardar en L1 (Moka)
        self.l1.insert_with_ttl(
            key.to_string(),
            value.clone(),
            l1_ttl,
        ).await;
        
        // 2. Guardar en L2 (Redis) si está disponible
        if let Some(ref redis) = self.l2 {
            redis.set(key, &value, l2_ttl.as_secs()).await?;
        }
        
        Ok(())
    }
}
```

### 4. Decorator pattern (compatible con ADR 0017)

```rust
// crates/cache/src/decorator.rs
use std::future::Future;
use std::pin::Pin;

pub struct CacheDecorator<T: Clone + Send + Sync> {
    cache: TieredCache<T>,
}

impl<T: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned> CacheDecorator<T> {
    pub async fn cached<F, Fut>(
        &self,
        key: &str,
        ttl_l1: std::time::Duration,
        ttl_l2: std::time::Duration,
        fetch: F,
    ) -> Result<T, CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, CacheError>>,
    {
        // 1. Intentar caché
        if let Some(value) = self.cache.get(key).await? {
            return Ok(value);
        }
        
        // 2. Miss → ejecutar fetch
        let value = fetch().await?;
        
        // 3. Guardar en caché
        self.cache.set(key, value.clone(), ttl_l1, ttl_l2).await?;
        
        Ok(value)
    }
}
```

---

## Use cases avanzados

### 1. Rate Limiting Distribuido

```rust
// crates/rate_limit/src/redis_backend.rs
use redis::AsyncCommands;

pub async fn check_rate_limit(
    redis: &mut deadpool_redis::Connection,
    key: &str,
    max_requests: u32,
    window_seconds: u64,
) -> Result<bool, RateLimitError> {
    let current: u32 = redis.incr(key).await?;
    
    if current == 1 {
        // Primera request, establecer TTL
        redis.expire(key, window_seconds).await?;
    }
    
    Ok(current <= max_requests)
}
```

### 2. Sesiones Compartidas

```rust
// sessions en Redis para múltiples API nodes
pub async fn store_session(
    redis: &mut deadpool_redis::Connection,
    session_id: &str,
    user_id: &str,
    ttl_hours: u64,
) -> Result<(), SessionError> {
    redis.set_ex(
        format!("session:{}", session_id),
        user_id,
        ttl_hours * 3600,
    ).await?;
    Ok(())
}
```

### 3. Pub/Sub para Eventos

```rust
// Notificaciones en tiempo real entre nodos
pub async fn subscribe_notifications(
    redis: &deadpool_redis::Pool,
    channel: &str,
) -> Result<impl Stream<Item = String>, PubSubError> {
    let mut conn = redis.get().await?;
    conn.subscribe(channel).await?;
    
    Ok(conn.into_on_message().map(|msg| {
        msg.get_payload::<String>().unwrap_or_default()
    }))
}
```

---

## Consecuencias

### ✅ Positivas

- **Caché compartida:** Todos los nodos ven la misma caché (consistencia)
- **Escalabilidad horizontal:** Agregar nodos API no fragmenta la caché
- **Persistencia:** Redis AOF/RDB sobrevive reinicios
- **Features avanzadas:** Pub/sub, rate limiting, leaderboards, geospatial
- **Atomicidad:** Operaciones atómicas en caché (INCR, HSET, etc.)

### ⚠️ Negativas / Trade-offs

- **Complejidad:** 6 contenedores (3 masters + 3 replicas) vs 0 (Moka)
- **Latencia:** 1ms vs 1μs (1000x más lento que Moka local)
- **Costo infraestructura:** $50-200/mes para Redis managed
- **Single point of failure:** Si Redis cae, la caché global desaparece
- **Serialización:** Todos los valores deben serializarse (JSON/bincode)
- **Despliegue:** Setup inicial complejo (clustering, sharding, failover)

### Decisiones derivadas

- Mantener Moka L1 SIEMPRE (incluso con Redis) para hot data
- Redis es L2 opcional — activar solo cuando >5 nodos API
- TTL diferenciado: L1 (60s), L2 (1h), DB (∞)
- Cache stampede protection: usar probabilistic early expiration

---

## Estado actual

**No implementar.** Mantener Moka (ADR 0017) hasta que:
1. Hay >5 réplicas del API necesitando caché compartida
2. La caché local excede 100MB por nodo
3. Se necesiten features avanzadas (pub/sub, rate limiting distribuido)

**Path recomendado:**
1. Fase 1-3: Moka solamente (10MB por nodo, local)
2. Fase 4: Redis simple (1 nodo, sin clustering)
3. Fase 5+: Redis Cluster (6 nodos, alta disponibilidad)

Ver ADR 0017 para Moka y ADR 0031 para criterios de escalamiento.
