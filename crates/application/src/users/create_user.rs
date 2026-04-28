use domain::entities::User;
use domain::errors::DomainError;
use domain::ports::{PasswordHasher, UserRepository};
use domain::value_objects::{Email, PasswordHash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserInput {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserOutput {
    pub user_id: String,
    pub email: String,
}

pub struct CreateUserUseCase<'a, R, H>
where
    R: UserRepository,
    H: PasswordHasher,
{
    user_repo: &'a R,
    password_hasher: &'a H,
}

impl<'a, R, H> CreateUserUseCase<'a, R, H>
where
    R: UserRepository,
    H: PasswordHasher,
{
    pub fn new(user_repo: &'a R, password_hasher: &'a H) -> Self {
        Self {
            user_repo,
            password_hasher,
        }
    }

    pub async fn execute(&self, input: CreateUserInput) -> Result<CreateUserOutput, DomainError> {
        let email = Email::new(&input.email)?;

        if self.user_repo.find_active_by_email(&email).await?.is_some() {
            return Err(DomainError::EmailAlreadyExists {
                email: input.email,
            });
        }

        let password_hash_str = self.password_hasher.hash_password(&input.password)?;
        let password_hash = PasswordHash::new(&password_hash_str)?;

        let user = User::new(email, password_hash, input.name)?;

        self.user_repo.save(&user).await?;

        Ok(CreateUserOutput {
            user_id: user.id.to_string(),
            email: user.email.value().to_string(),
        })
    }
}