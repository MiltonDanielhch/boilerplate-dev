// Ubicación: `crates/application/src/use_cases.rs`
//
// Descripción: Casos de uso del sistema.
//
// ADRs relacionados: ADR 0001

use domain::entities::User;

pub struct CreateUserUseCase;

impl CreateUserUseCase {
    pub fn execute(&self, email: String) -> User {
        User::new(email)
    }
}
