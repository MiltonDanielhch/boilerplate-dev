// Ubicación: `crates/auth/src/paseto.rs`
//
// Descripción: Servicio PASETO v4 Local para access tokens.
//              NUNCA genera tokens con "eyJ" (JWT está PROHIBIDO — ADR 0008).
//
// ADRs relacionados: ADR 0008, ADR 0002 (fail-fast)

use domain::errors::DomainError;
use pasetors::claims::{Claims, ClaimsValidationRules};
use pasetors::keys::SymmetricKey;
use pasetors::local;
use pasetors::token::UntrustedToken;
use pasetors::{Local, version4::V4};
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Claims personalizados para nuestros tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject (user_id)
    pub sub: String,
    /// Token ID (único por token)
    pub jti: String,
    /// Issued at
    pub iat: i64,
    /// Expiration
    pub exp: i64,
}

/// Servicio para generar y verificar tokens PASETO v4 Local.
pub struct PasetoService {
    key: SecretBox<[u8; 32]>,
}

impl Clone for PasetoService {
    fn clone(&self) -> Self {
        // Copiar los bytes de la clave secreta
        let key_copy = *self.key.expose_secret();
        Self {
            key: SecretBox::new(Box::new(key_copy)),
        }
    }
}

impl PasetoService {
    /// Crea un nuevo PasetoService.
    /// Panic si el secreto no tiene exactamente 32 bytes (fail-fast intencional).
    /// Ref: ADR 0002
    pub fn new(secret: &str) -> Self {
        let secret_bytes = hex::decode(secret)
            .expect("PASETO_SECRET debe ser hex válido (fail-fast)");

        if secret_bytes.len() != 32 {
            panic!(
                "PASETO_SECRET debe tener exactamente 32 bytes ({} encontrados). \
                 Use: openssl rand -hex 32",
                secret_bytes.len()
            );
        }

        // Convertir Vec<u8> a [u8; 32]
        let key_bytes: [u8; 32] = secret_bytes.try_into()
            .expect("PASETO_SECRET debe tener exactamente 32 bytes");
        
        Self {
            key: SecretBox::new(Box::new(key_bytes)),
        }
    }

    /// Genera un access token PASETO v4 válido por 15 minutos.
    /// El token SIEMPRE empieza con "v4.local."
    /// Ref: docs/02-STACK.md L213-217, ADR 0008
    pub fn generate_access_token(&self, user_id: &Uuid) -> Result<String, DomainError> {
        let now = OffsetDateTime::now_utc();
        let expiration = now + Duration::minutes(15);

        let mut claims = Claims::new().map_err(|e| {
            DomainError::Internal(format!("Failed to create claims: {}", e))
        })?;

        claims.subject(&user_id.to_string());
        claims.token_identifier(&Uuid::new_v4().to_string());
        claims.issued_at(&now.format(&time::format_description::well_known::Rfc3339).unwrap());
        claims.expiration(&expiration.format(&time::format_description::well_known::Rfc3339).unwrap());

        let key = SymmetricKey::<V4>::from(self.key.expose_secret().as_slice())
            .map_err(|e| DomainError::Internal(format!("Invalid PASETO key: {}", e)))?;

        let token = local::encrypt(&key, &claims, None, None)
            .map_err(|e| DomainError::Internal(format!("Token encryption failed: {}", e)))?;

        // Verificación de seguridad: el token DEBE empezar con "v4.local."
        if !token.starts_with("v4.local.") {
            return Err(DomainError::Internal(
                "Token generation violation: NOT v4.local".to_string()
            ));
        }

        Ok(token)
    }

    /// Verifica un token PASETO v4 y retorna los claims.
    /// Retorna DomainError::InvalidToken si el token es inválido o expiró.
    pub fn verify(&self, token: &str) -> Result<TokenClaims, DomainError> {
        // Validación rápida: debe empezar con v4.local.
        if !token.starts_with("v4.local.") {
            return Err(DomainError::InvalidToken);
        }

        // JWT está prohibido — detectar tokens que empiecen con "eyJ"
        if token.starts_with("eyJ") {
            tracing::error!("JWT token detected — ADR 0008 violation");
            return Err(DomainError::InvalidToken);
        }

        let key = SymmetricKey::<V4>::from(self.key.expose_secret().as_slice())
            .map_err(|e| {
                tracing::error!("Invalid PASETO key: {}", e);
                DomainError::InvalidToken
            })?;

        let validation_rules = ClaimsValidationRules::new();
        
        // API correcta: UntrustedToken::<Purpose, Version>
        let untrusted: UntrustedToken<Local, V4> = token.try_into()
            .map_err(|_| DomainError::InvalidToken)?;

        let trusted = local::decrypt(&key, &untrusted, &validation_rules, None, None)
            .map_err(|e| {
                tracing::debug!("Token decryption failed: {}", e);
                DomainError::InvalidToken
            })?;

        // Extraer claims del payload JSON del token
        let claims = trusted.payload_claims()
            .ok_or(DomainError::InvalidToken)?;
        let payload_json = serde_json::to_string(claims)
            .map_err(|_| DomainError::InvalidToken)?;
        
        let claims_value: serde_json::Value = serde_json::from_str(&payload_json)
            .map_err(|_| DomainError::InvalidToken)?;

        let sub = claims_value["sub"]
            .as_str()
            .ok_or(DomainError::InvalidToken)?
            .to_string();

        let jti = claims_value["jti"]
            .as_str()
            .ok_or(DomainError::InvalidToken)?
            .to_string();

        let iat_str = claims_value["iat"]
            .as_str()
            .ok_or(DomainError::InvalidToken)?;
        let iat = OffsetDateTime::parse(iat_str, &time::format_description::well_known::Rfc3339)
            .map_err(|_| DomainError::InvalidToken)?
            .unix_timestamp();

        let exp_str = claims_value["exp"]
            .as_str()
            .ok_or(DomainError::InvalidToken)?;
        let exp = OffsetDateTime::parse(exp_str, &time::format_description::well_known::Rfc3339)
            .map_err(|_| DomainError::InvalidToken)?
            .unix_timestamp();

        Ok(TokenClaims { sub, jti, iat, exp })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_test_secret() -> String {
        // Secreto de prueba: 32 bytes hex = 64 caracteres
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()
    }

    #[test]
    fn test_paseto_service_new_valid_secret() {
        let secret = generate_test_secret();
        let _service = PasetoService::new(&secret);
    }

    #[test]
    #[should_panic(expected = "PASETO_SECRET debe tener exactamente 32 bytes")]
    fn test_paseto_service_new_invalid_secret_length() {
        let short_secret = "0123456789abcdef"; // Solo 16 bytes
        let _service = PasetoService::new(short_secret);
    }

    #[test]
    #[should_panic(expected = "PASETO_SECRET debe ser hex válido")]
    fn test_paseto_service_new_invalid_hex() {
        let invalid_hex = "not_valid_hex!!!".to_string();
        let _service = PasetoService::new(&invalid_hex);
    }

    #[test]
    fn test_generate_and_verify_token() {
        let secret = generate_test_secret();
        let service = PasetoService::new(&secret);
        let user_id = Uuid::new_v4();

        // Generar token
        let token = service.generate_access_token(&user_id).expect("Should generate token");

        // Verificar formato v4.local.
        assert!(token.starts_with("v4.local."));
        assert!(!token.starts_with("eyJ")); // NO es JWT

        // Verificar token
        let claims = service.verify(&token).expect("Should verify token");
        assert_eq!(claims.sub, user_id.to_string());
        assert!(!claims.jti.is_empty());

        // El token debe expirar en el futuro
        let now = OffsetDateTime::now_utc().unix_timestamp();
        assert!(claims.exp > now);
    }

    #[test]
    fn test_verify_invalid_token() {
        let secret = generate_test_secret();
        let service = PasetoService::new(&secret);

        let result = service.verify("invalid_token");
        assert!(matches!(result, Err(DomainError::InvalidToken)));
    }

    #[test]
    fn test_verify_jwt_token_rejected() {
        let secret = generate_test_secret();
        let service = PasetoService::new(&secret);

        // Intentar verificar un token JWT (debe ser rechazado)
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlWFh3N8n8";

        let result = service.verify(jwt);
        assert!(matches!(result, Err(DomainError::InvalidToken)));
    }

    #[test]
    fn test_verify_tampered_token() {
        let secret = generate_test_secret();
        let service = PasetoService::new(&secret);
        let user_id = Uuid::new_v4();

        let token = service.generate_access_token(&user_id).expect("Should generate");
        let tampered = format!("{}tampered", token);

        let result = service.verify(&tampered);
        assert!(matches!(result, Err(DomainError::InvalidToken)));
    }
}
