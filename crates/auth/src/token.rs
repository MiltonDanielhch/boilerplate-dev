// Ubicación: `crates/auth/src/token.rs`
//
// Descripción: Tokens PASETO v4 (NO JWT).
//
// ADRs relacionados: ADR 0008

pub fn generate_token(user_id: &str) -> String {
    // TODO: Implementar con pasetors
    format!("v4.local.{}", user_id)
}
