// Ubicación: `crates/application/src/auth/mod.rs`
//
// Descripción: Casos de uso de autenticación (auth).
//
// ADRs relacionados: ADR 0001, ADR 0008 (PASETO)

pub mod login;
pub mod logout;
pub mod refresh;
pub mod register;

// Re-exports
pub use login::LoginUseCase;
pub use logout::LogoutUseCase;
pub use refresh::RefreshUseCase;
pub use register::RegisterUseCase;
