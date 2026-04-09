// Ubicación: `crates/storage/src/ports.rs`
//
// Descripción: Puerto (trait) de storage — define el contrato.
//
// ADRs relacionados: ADR 0001

pub trait StoragePort {
    fn upload(
        &self,
        key: &str,
        data: &[u8],
    ) -> impl std::future::Future<Output = Result<String, String>> + Send;
}
