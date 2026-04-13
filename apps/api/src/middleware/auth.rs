// Ubicación: `apps/api/src/middleware/auth.rs`
//
// Descripción: Middleware de autenticación — extrae Bearer token y verifica PASETO v4.
//
// ADRs relacionados: ADR 0008 (PASETO), ADR 0006 (RBAC)

use crate::error::ApiError;
use crate::state::AppState;
use auth::TokenClaims;
use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};

/// Claims extraídas del token PASETO v4 validado.
#[derive(Debug, Clone)]
pub struct AuthClaims {
    pub user_id: String,
    pub token_id: String,
    pub issued_at: i64,
    pub expires_at: i64,
}

impl From<TokenClaims> for AuthClaims {
    fn from(claims: TokenClaims) -> Self {
        Self {
            user_id: claims.sub,
            token_id: claims.jti,
            issued_at: claims.iat,
            expires_at: claims.exp,
        }
    }
}

/// Extractor para obtener claims autenticadas en handlers.
/// 
/// Uso:
/// ```rust
/// pub async fn handler(
///     claims: AuthClaims,  // Extrae automáticamente del token
/// ) { ... }
/// ```
impl<S> FromRequestParts<S> for AuthClaims
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Buscar claims en extensions (inyectadas por middleware)
        parts
            .extensions
            .get::<AuthClaims>()
            .cloned()
            .ok_or_else(|| ApiError::Unauthorized("Missing authentication".to_string()))
    }
}

/// Middleware que extrae y valida el Bearer token del header Authorization.
/// 
/// Si el token es válido, inyecta `AuthClaims` en las extensions del request.
/// Si no, retorna 401 Unauthorized.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extraer header Authorization
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Err(ApiError::Unauthorized(
                "Missing or invalid Authorization header".to_string()
            ));
        }
    };

    // Verificar token PASETO v4
    let claims = state
        .paseto
        .verify(token)
        .map_err(|_| ApiError::Unauthorized("Invalid or expired token".to_string()))?;

    // Convertir a AuthClaims e inyectar en extensions
    let auth_claims = AuthClaims::from(claims);
    request.extensions_mut().insert(auth_claims);

    // Continuar con el request
    Ok(next.run(request).await)
}

/// Middleware opcional de autenticación.
/// 
/// Si hay token válido, inyecta claims. Si no, continúa sin claims.
/// Útil para endpoints que funcionan con o sin auth.
pub async fn optional_auth_middleware(
    State(state): State<AppState>,
    mut request: axum::extract::Request,
    next: Next,
) -> Response {
    // Extraer header Authorization
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    // Si hay token válido, inyectar claims
    if let Some(header) = auth_header {
        if header.starts_with("Bearer ") {
            let token = &header[7..];
            if let Ok(claims) = state.paseto.verify(token) {
                let auth_claims = AuthClaims::from(claims);
                request.extensions_mut().insert(auth_claims);
            }
        }
    }

    // Continuar siempre (con o sin claims)
    next.run(request).await
}
