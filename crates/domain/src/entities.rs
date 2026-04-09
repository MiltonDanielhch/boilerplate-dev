// Ubicación: `crates/domain/src/entities.rs`
//
// Descripción: Entidades del dominio. Ejemplo: User.
//
// ADRs relacionados: ADR 0001

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
}

impl User {
    pub fn new(email: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
        }
    }
}
