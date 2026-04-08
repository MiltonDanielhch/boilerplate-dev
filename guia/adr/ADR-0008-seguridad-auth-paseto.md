# ADR 0008 — Seguridad: argon2id + PASETO v4 Local + Refresh Tokens

| Campo | Valor |
|-------|-------|
| **Estado** | ✅ Aceptado |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0003 (Axum middleware), ADR 0016 (Tracing / request_id), ADR 0004 (SQLite para refresh tokens) |

---

## Contexto

JWT presenta riesgos de seguridad conocidos: "agilidad de algoritmos" (el header `alg` puede
manipularse para degradar a algoritmos débiles o `none`) y payloads visibles en Base64 sin cifrado.

Necesitamos autenticación que sea:

- **Segura por defecto** — sin opciones de configuración incorrecta posibles
- **Payload privado** — el `user_id` no debe ser legible por intermediarios
- **Soberana** — sin dependencia de servicios externos de identidad

**JWT está prohibido en este proyecto.** `jsonwebtoken` no puede aparecer en ningún
`Cargo.toml` del workspace.

---

## Decisión

Usar **argon2id** para hashing de contraseñas, **PASETO v4 Local** para access tokens,
y **refresh tokens opacos** almacenados en SQLite con rotación obligatoria.

### 1 — Hash de contraseñas: argon2id

Parámetros OWASP 2024 — máxima resistencia a ataques de GPU/ASIC:

```toml
# crates/auth/Cargo.toml
argon2   = "0.5"
pasetors = { version = "0.7", features = ["v4"] }
# jsonwebtoken — NUNCA. JWT está prohibido en este proyecto.
```

```rust
// crates/auth/src/password.rs
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt   = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Parámetros OWASP por defecto
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AuthError::HashingFailed)?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed = PasswordHash::new(hash).map_err(|_| AuthError::InvalidHash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
```

### 2 — Access Token: PASETO v4 Local

A diferencia de JWT, el payload está **cifrado simétricamente** con XChaCha20-Poly1305.
No hay cabecera `alg` — el algoritmo es fijo e inamovible:

```rust
// crates/auth/src/paseto.rs
use pasetors::v4::local::{LocalKey, encode, decode};
use pasetors::claims::Claims;

pub struct PasetoService {
    key: LocalKey, // Desde PASETO_SECRET (exactamente 32 bytes)
}

impl PasetoService {
    /// Panics si el secret no tiene exactamente 32 bytes — fail-fast intencional
    pub fn new(secret: &str) -> Self {
        assert!(
            secret.len() == 32,
            "PASETO_SECRET debe tener exactamente 32 bytes, tiene {}",
            secret.len()
        );
        let key = LocalKey::from(secret.as_bytes().try_into().unwrap());
        Self { key }
    }

    pub fn generate_access_token(&self, user_id: &str) -> Result<String, AuthError> {
        let mut claims = Claims::new().map_err(|_| AuthError::Internal)?;
        claims.add_additional("sub", user_id).map_err(|_| AuthError::Internal)?;
        claims.expiration(&(OffsetDateTime::now_utc() + Duration::minutes(15)))
            .map_err(|_| AuthError::Internal)?;

        encode(
            &self.key,
            &claims,
            None,
            Some(b"boilerplate-v1"), // Implicit assertion — enlaza el token al sistema
        )
        .map_err(|_| AuthError::TokenCreation)
    }

    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        decode(&self.key, token, None, Some(b"boilerplate-v1"))
            .map_err(|_| AuthError::Unauthorized)
    }
}
```

El token generado empieza siempre con `v4.local.` — nunca con `eyJ` (JWT).

### 3 — Refresh Token: estado en SQLite

Tokens opacos de 32 bytes con rotación obligatoria:

```sql
-- Las 6 migraciones del sistema (ADR 0006) ya incluyen esta tabla
CREATE TABLE tokens (
    id         TEXT     PRIMARY KEY NOT NULL,
    user_id    TEXT     NOT NULL,
    token_hash TEXT     NOT NULL UNIQUE,
    type       TEXT     NOT NULL,  -- "refresh" | "email_verification" | "password_reset"
    expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    used       BOOLEAN  DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

### 4 — Middleware de autenticación (Axum)

```rust
// crates/infrastructure/src/http/middleware/auth.rs
pub async fn auth_middleware(
    State(paseto): State<Arc<PasetoService>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims  = paseto.verify(token).map_err(|_| AppError::Unauthorized)?;
    let user_id = claims
        .get_claim("sub")
        .and_then(|v| v.as_str())
        .ok_or(AppError::Unauthorized)?;

    // Registrar el user_id en el span de tracing (ADR 0016)
    tracing::Span::current().record("user_id", user_id);
    req.extensions_mut().insert(UserId(user_id.to_string()));

    Ok(next.run(req).await)
}
```

### Por qué PASETO en lugar de JWT

| JWT | PASETO v4 Local |
|-----|-----------------|
| Header `alg` manipulable (`alg: none`) | Sin header de algoritmo — XChaCha20-Poly1305 fijo |
| Payload visible en Base64 | Payload **cifrado** — user_id invisible para intermediarios |
| Múltiples algoritmos = superficie de ataque | Un solo algoritmo = imposible degradar |
| `jsonwebtoken` crate bien conocido | `pasetors` — diseñado específicamente para PASETO |

---

## Herramientas y Librerías para Optimizar (Edición 2026)

Para fortalecer la criptografía y la integración con el framework:

| Herramienta | Propósito en la Seguridad |
| :--- | :--- |
| **`secrecy`** | **Higiene de Claves:** Asegura que la clave de PASETO no se filtre accidentalmente en logs o volcados de memoria. |
| **`paseto`** | **Motor de Tokens:** Alternativa ergonómica a `pasetors` para el manejo de claims y cifrado v4.local. |
| **`constant_time_eq`** | **Prevención de Timing Attacks:** Comparación segura de hashes de tokens para evitar fugas de información. |
| **`argon2`** | **Hashing de Clase Mundial:** Implementación de referencia para argon2id con parámetros OWASP 2024. |

---

## Alternativas consideradas

| Opción | Motivo de descarte |
|--------|--------------------|
| JWT | Vulnerable a manipulación de cabecera; payload visible; prohibido en este proyecto |
| Cookies de sesión puras | Requieren stickiness o DB compartida para escalar |
| Auth0 / Clerk | Dependencia externa — contradice el principio de soberanía del proyecto |

---

## Consecuencias

### ✅ Positivas

- Criptografía moderna — XChaCha20-Poly1305 para cifrado, argon2id para contraseñas
- Payload privado — el `user_id` no es visible para proxies ni sniffers
- Sin opciones inseguras — PASETO no permite elegir algoritmos; si el token no es v4, no se valida
- Rotación de refresh tokens — cada uso emite un token nuevo, el anterior queda revocado

### ⚠️ Negativas / Trade-offs

- Menos herramientas de debugging visual que JWT (no hay equivalente a `jwt.io` para PASETO)
  → Para inspeccionar un token PASETO en desarrollo:
    `pasetors` tiene un modo de decode sin verificación para debugging local
  → En producción: el `request_id` en los logs es suficiente para rastrear sesiones
- El secreto `PASETO_SECRET` debe ser exactamente 32 bytes
  → Mitigación: `PasetoService::new()` hace panic si no tiene 32 bytes — falla al arrancar,
    no en runtime. Nunca llega a producción con un secreto incorrecto.
  → Generar un secreto correcto: `openssl rand -hex 16` (produce 32 bytes en hex)
- Ecosistema de librerías PASETO más pequeño que JWT en algunos lenguajes del cliente
  → Para Kotlin (Android) y Swift (iOS): hay bibliotecas PASETO maduras disponibles
  → Para clientes TypeScript: `@noble/ciphers` o `paseto` npm package

### Decisiones derivadas

- `PASETO_SECRET` se valida en el arranque de la app con panic explícito
- `jsonwebtoken` está prohibido — `sintonia check arch` (ADR 0028) lo detecta automáticamente
- El `CleanupJob` (ADR 0018) limpia refresh tokens expirados y tokens usados
- El `user_id` se registra en cada span de tracing como campo indexado (ADR 0016)
