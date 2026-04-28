// Ubicación: `crates/application/src/users/mod.rs`
//
// Descripción: Casos de uso de gestión de usuarios.
//
// ADRs relacionados: ADR 0001, ADR 0006 (Soft Delete)

pub mod create_user;
pub mod get_user;
pub mod impersonate_user;
pub mod list_users;
pub mod soft_delete_user;
pub mod update_user;

// Re-exports
pub use create_user::{CreateUserUseCase, CreateUserInput, CreateUserOutput};
pub use get_user::GetUserUseCase;
pub use impersonate_user::{ImpersonateUserUseCase, ImpersonateUserInput};
pub use list_users::{ListUsersUseCase, ListUsersInput};
pub use soft_delete_user::SoftDeleteUserUseCase;
pub use update_user::{UpdateUserUseCase, UpdateUserInput};
