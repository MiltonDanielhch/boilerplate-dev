// Ubicación: `crates/database/src/repositories/cached_user_repository.rs`
//
// Descripción: Decorador de caché para UserRepository usando Moka.
//              Implementa ADR 0017.
//
// Cache configurado:
// - TTL: 5 minutos
// - Max capacity: 10,000 entries
// - TTI (time to idle): 1 minuto
//
// CRÍTICO: Invalidación en save() y soft_delete()

use crate::repositories::sqlite_user_repository::SqliteUserRepository;
use domain::entities::User;
use domain::errors::DomainError;
use domain::ports::UserRepository;
use domain::value_objects::{Email, UserId};
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

pub struct CachedUserRepository {
    inner: Arc<SqliteUserRepository>,
    cache: Cache<UserId, User>,
    email_cache: Cache<String, UserId>,
}

impl Clone for CachedUserRepository {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            cache: self.cache.clone(),
            email_cache: self.email_cache.clone(),
        }
    }
}

impl CachedUserRepository {
    pub fn new(inner: SqliteUserRepository) -> Self {
        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300)) // 5 min TTL
            .time_to_idle(Duration::from_secs(60))  // 1 min TTI
            .build();

        let email_cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300))
            .time_to_idle(Duration::from_secs(60))
            .build();

        Self {
            inner: Arc::new(inner),
            cache,
            email_cache,
        }
    }

    fn cache_key(id: &UserId) -> String {
        id.to_string()
    }

    async fn invalidate(&self, user: &User) {
        let id_key = Self::cache_key(&user.id);
        self.cache.invalidate(&user.id).await;
        
        let email_str = user.email.value().to_string();
        self.email_cache.invalidate(&email_str).await;
        
        tracing::debug!(user_id = %id_key, "Cache invalidated for user");
    }
}

impl UserRepository for CachedUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        let key = Self::cache_key(id);
        
        if let Some(cached) = self.cache.get(id).await {
            tracing::debug!(user_id = %key, "L1 HIT");
            return Ok(Some(cached));
        }
        
        tracing::debug!(user_id = %key, "L1 MISS");
        let result = self.inner.find_by_id(id).await?;
        
        if let Some(user) = &result {
            self.cache.insert(user.id.clone(), user.clone()).await;
        }
        
        Ok(result)
    }

    async fn find_active_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let email_key = email.value();
        
        if let Some(user_id) = self.email_cache.get(email_key).await {
            if let Some(user) = self.cache.get(&user_id).await {
                tracing::debug!(email = %email_key, "L1 HIT (via email cache)");
                return Ok(Some(user));
            }
        }
        
        tracing::debug!(email = %email_key, "L1 MISS (email cache)");
        let result = self.inner.find_active_by_email(email).await?;
        
        if let Some(user) = &result {
            self.email_cache.insert(email_key.to_string(), user.id.clone()).await;
            self.cache.insert(user.id.clone(), user.clone()).await;
        }
        
        Ok(result)
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        self.invalidate(user).await;
        self.inner.save(user).await
    }

    async fn soft_delete(&self, id: &UserId) -> Result<(), DomainError> {
        if let Ok(Some(user)) = self.inner.find_by_id(id).await {
            self.invalidate(&user).await;
        }
        self.cache.invalidate(id).await;
        self.inner.soft_delete(id).await
    }

    async fn has_permission(&self, user_id: &UserId, permission: &str) -> Result<bool, DomainError> {
        self.inner.has_permission(user_id, permission).await
    }

    async fn get_permissions(&self, user_id: &UserId) -> Result<Vec<String>, DomainError> {
        self.inner.get_permissions(user_id).await
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>, DomainError> {
        self.inner.list(limit, offset).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::value_objects::PasswordHash;

    fn test_user() -> User {
        let email = Email::new("test@example.com").unwrap();
        let hash = PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$hash").unwrap();
        User::new(email, hash, Some("Test User".to_string())).unwrap()
    }

    #[tokio::test]
    async fn cache_hit_after_save() {
        let repo = CachedUserRepository::new(SqliteUserRepository::new(
            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .unwrap()
        ));

        let user = test_user();
        repo.save(&user).await.unwrap();

        let found = repo.find_by_id(&user.id).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn cache_miss_then_hit() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
            
        let inner = SqliteUserRepository::new(pool);
        let repo = CachedUserRepository::new(inner);

        let user = test_user();
        let id = user.id.clone();

        let miss = repo.find_by_id(&id).await.unwrap();
        assert!(miss.is_none());

        repo.save(&user).await.unwrap();

        let hit = repo.find_by_id(&id).await.unwrap();
        assert!(hit.is_some());
    }

    #[tokio::test]
    async fn cache_invalidated_after_soft_delete() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
            
        let inner = SqliteUserRepository::new(pool);
        let repo = CachedUserRepository::new(inner);

        let user = test_user();
        let id = user.id.clone();

        repo.save(&user).await.unwrap();
        
        let cached_before = repo.find_by_id(&id).await.unwrap();
        assert!(cached_before.is_some());

        repo.soft_delete(&id).await.unwrap();

        let after_delete = repo.find_by_id(&id).await.unwrap();
        assert!(after_delete.is_none() || !after_delete.unwrap().is_active());
    }
}