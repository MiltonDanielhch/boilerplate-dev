// Ubicación: `crates/domain/src/value_objects.rs`
//
// Descripción: Value objects del dominio. Inmutables y validados.
//
// ADRs relacionados: ADR 0001

#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn new(value: &str) -> Result<Self, String> {
        if value.contains('@') {
            Ok(Self(value.to_string()))
        } else {
            Err("Email inválido".to_string())
        }
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
