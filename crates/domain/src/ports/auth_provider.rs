// Ubicación: `crates/domain/src/ports/auth_provider.rs`
//
// Descripción: Puertos para la generación de tokens y verificación de contraseñas.
//              Implementado por `crates/auth`.



pub trait PasswordVerifier: Send + Sync {
    /// Verifica que el hash coincida con la contraseña en texto plano
    fn verify_password(&self, plain: &str, hash: &str) -> Result<bool, crate::errors::DomainError>;
}

pub trait PasswordHasher: Send + Sync {
    /// Hashea una contraseña usando Argon2id
    fn hash_password(&self, plain: &str) -> Result<String, crate::errors::DomainError>;
}

pub trait TokenGenerator: Send + Sync {
    /// Genera un token de acceso (PASETO v4)
    fn generate_access_token(&self, user_id: &uuid::Uuid) -> Result<String, crate::errors::DomainError>;
    
    /// Genera un token de refresco opaco (aleatorio seguro)
    fn generate_refresh_token(&self) -> String;
    
    /// Aplica un hash unidireccional al token de refresco para guardarlo en DB
    fn hash_refresh_token(&self, token: &str) -> String;
}
