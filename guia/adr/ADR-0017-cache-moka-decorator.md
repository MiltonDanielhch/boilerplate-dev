# ADR 0017 — Caché In-Process: Moka + Patrón Decorator

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (Persistencia SQLite), ADR 0025 (NATS para invalidación L2 en Fase 2) |

---

## Contexto

El sistema opera en un entorno de recursos limitados — VPS de $5 con 1GB de RAM. El acceso
a disco con SQLite, aunque rápido, introduce latencia y contención en escenarios de alta
lectura (90% de las operaciones).

Necesitamos una solución que:

- **Elimine la latencia de I/O** — evitar consultas repetitivas a DB para datos estáticos o de sesión
- **Arquitectura limpia** — los casos de uso del dominio no deben saber que existe un caché
- **Seguridad de tipos** — almacenar structs de Rust directamente en memoria, sin serialización

---

## Decisión

Usar **Moka** como motor de caché in-memory, aplicando el **Patrón Decorator** en la capa
de infraestructura para envolver los repositorios sin contaminar el dominio.

### Dependencia

```toml
# crates/database/Cargo.toml
moka = { version = "0.12", features = ["future"] }
```

### El contrato del dominio (no cambia)

El puerto permanece agnóstico al caché — el dominio no sabe que existe:

```rust
// crates/domain/src/ports/user_repository.rs
pub trait UserRepository: Send + Sync {
    async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;
    async fn save(&self, user: &User)                   -> Result<(), DomainError>;
    async fn soft_delete(&self, id: &UserId)            -> Result<(), DomainError>;
}
```

### El decorador de caché (crates/database)

```rust
// crates/database/src/repositories/cached_user_repository.rs
pub struct CachedUserRepository<R: UserRepository> {
    inner: R,                          // El repositorio real (SQLite)
    cache: Cache<String, User>,
}

impl<R: UserRepository> CachedUserRepository<R> {
    pub fn new(inner: R) -> Self {
        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300))   // TTL: 5 minutos
            .time_to_idle(Duration::from_secs(60))    // TTI: 1 min sin acceso
            .build();
        Self { inner, cache }
    }
}

impl<R: UserRepository> UserRepository for CachedUserRepository<R> {
    async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let key = email.as_str().to_string();

        // 1. Cache HIT
        if let Some(user) = self.cache.get(&key).await {
            tracing::debug!(email = %email, "caché L1 HIT");
            return Ok(Some(user));
        }

        // 2. Cache MISS — ir a la base de datos
        tracing::debug!(email = %email, "caché L1 MISS — consultando DB");
        let user = self.inner.find_active_by_email(email).await?;

        // 3. Populate — guardar en caché si existe
        if let Some(ref u) = user {
            self.cache.insert(key, u.clone()).await;
        }

        Ok(user)
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        let result = self.inner.save(user).await?;
        // Invalidación proactiva OBLIGATORIA — sin esto hay datos obsoletos
        self.cache.invalidate(&user.email.as_str().to_string()).await;
        tracing::debug!(email = %user.email, "caché L1 invalidado tras save");
        Ok(result)
    }

    async fn soft_delete(&self, id: &UserId) -> Result<(), DomainError> {
        // Buscar el user para obtener el email antes de borrar
        if let Ok(Some(user)) = self.inner.find_by_id(id).await {
            self.inner.soft_delete(id).await?;
            self.cache.invalidate(&user.email.as_str().to_string()).await;
            tracing::debug!(user_id = %id, "caché L1 invalidado tras soft_delete");
        }
        Ok(())
    }
}
```

### Test de invalidación — obligatorio

```rust
#[tokio::test]
async fn cache_se_invalida_tras_soft_delete() {
    let pool = setup_test_db().await;
    let repo = CachedUserRepository::new(SqliteUserRepository::new(pool));
    let email = Email::new("test@example.com").unwrap();

    // 1. Guardar
    let user = User::new(email.clone(), PasswordHash::from_hash("h".into()));
    repo.save(&user).await.unwrap();

    // 2. Primera lectura — va a DB (MISS)
    let found = repo.find_active_by_email(&email).await.unwrap();
    assert!(found.is_some());

    // 3. Segunda lectura — viene del caché (HIT)
    let cached = repo.find_active_by_email(&email).await.unwrap();
    assert!(cached.is_some());

    // 4. Soft delete — invalida el caché
    repo.soft_delete(&user.id).await.unwrap();

    // 5. Lectura post-delete — debe ir a DB y retornar None (usuario inactivo)
    let after_delete = repo.find_active_by_email(&email).await.unwrap();
    assert!(after_delete.is_none()); // Si el caché no se invalidó, esto falla
}
```

### Inyección de dependencias en el composition root

```rust
// apps/api/src/setup.rs
let real_repo   = SqliteUserRepository::new(pool.clone());
let cached_repo = CachedUserRepository::new(real_repo);

// AppState solo ve un UserRepository — sin conocer el caché
let state = AppState {
    user_repo: Arc::new(cached_repo),
};
```

### Convenciones de expiración e invalidación

| Tipo de dato | TTL | TTI | Invalidación |
|-------------|-----|-----|-------------|
| Usuarios activos | 5 minutos | 1 minuto | Proactiva en save/soft_delete |
| Datos de sesión | 15 minutos | 2 minutos | Al logout o rotación de token |
| Configuración del sistema | 60 minutos | — | Solo en reinicio o deploy |

**Límite de RAM:** 100MB máximo para todos los cachés del sistema — garantiza estabilidad en VPS de 1GB.

---

## Path de escalado: L1 local → L2 distribuido

Cuando el proyecto pase a múltiples instancias, se agrega un segundo decorador sin tocar el dominio:

```
CachedUserRepository (Moka L1 — microsegundos, in-process)
    ↓ envuelve a
DistributedCacheRepository (Redis/NATS L2 — sincronización entre nodos)
    ↓ envuelve a
SqliteUserRepository (persistencia)
```

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| Redis desde el inicio | Proceso externo — overhead innecesario en MVP con una sola instancia |
| Cache en el handler de Axum | Contamina la capa HTTP con lógica de infraestructura |
| DashMap manual | Sin TTL ni eviction automática — hay que reimplementar lo que Moka ya da |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para maximizar la eficiencia y visibilidad del caché in-memory:

| Herramienta | Propósito en el Caché |
| :--- | :--- |
| **`moka` (future)** | **Motor de Caché:** Implementación de alto rendimiento con soporte nativo para `async/await`. |
| **`metrics`** | **Observabilidad:** Permite medir la tasa de aciertos (hit rate) para ajustar el tamaño de la RAM dinámicamente. |
| **`get_with` API** | **Prevenir Race Conditions:** Evita que múltiples hilos consulten la DB por la misma clave simultáneamente. |
| **`tracing`** | **Debugging:** Logs detallados de hit/miss integrados con el `request_id` (ADR 0016). |

---

## Consecuencias

### ✅ Positivas

- Los casos de uso son 100% legibles — sin ruido de caché en la lógica de negocio
- Lecturas repetidas no tocan el disco ni la red
- Activar o desactivar el caché es cambiar una línea en el composition root
- Patrón extensible — se puede agregar L2 sin modificar nada existente

### ⚠️ Negativas / Trade-offs

- Las entidades del dominio deben implementar `Clone`
  → Es un requisito mínimo y explícito en Rust — no introduce complejidad accidental
- En un VPS de 1GB hay que ser conservador con `max_capacity` para evitar OOM
  → El límite de 100MB total para cachés está configurado como variable de entorno
  → `max_capacity(10_000)` con entidades promedio de ~1KB = ~10MB por caché — dentro del límite
- La invalidación manual en métodos de escritura requiere disciplina
  → Mitigación: el test de invalidación en la sección anterior es obligatorio para cada
    método de escritura — falla el CI si se olvida
  → Si un método de escritura olvida invalidar: los datos obsoletos persisten hasta que
    el TTL de 5 minutos expira — aceptable como degradación controlada

### Decisiones derivadas

- Todo método de escritura (`save`, `update`, `soft_delete`) en el decorador debe llamar a
  `cache.invalidate()` inmediatamente después de que la DB confirme
- Se implementarán decoradores equivalentes para otras entidades de alta lectura
- El límite de 100MB de RAM para caché se configura como variable de entorno (`CACHE_MAX_MB`)
