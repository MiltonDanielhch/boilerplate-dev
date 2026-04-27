// Ubicación: `crates/mobile-bridge/src/lib.rs`
// 
// Descripción: Bridge de UniFFI para exponer lógica de Rust a Kotlin (Android)
//              y Swift (iOS) de forma nativa.
// 
// ADRs relacionados: 0030 (Multiplataforma), 0031 (Escalamiento)

uniffi::setup_scaffolding!("mobile_bridge");

#[uniffi::export]
pub fn get_version() -> String {
    "0.1.0-mobile-bridge".to_string()
}

#[derive(uniffi::Record)]
pub struct MobileAuthResult {
    pub success: bool,
    pub token: Option<String>,
    pub error: Option<String>,
}

#[uniffi::export]
pub async fn validate_email_natively(email: String) -> bool {
    domain::Email::new(&email).is_ok()
}
