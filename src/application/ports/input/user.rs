use async_trait::async_trait;
use thiserror::Error;

use crate::domain::user::User;

#[async_trait]
pub trait UserUseCase: Send + Sync + 'static {
    async fn register_user(&self, user: User, passwd: &str) -> Result<(), UserUseCaseError>;
    async fn verify_user_account(&self, token: &str) -> Result<(), UserUseCaseError>;
}

#[derive(Error, Debug)]
pub enum UserUseCaseError {
    #[error("The username provided is already in use.")]
    UsernameInUse,
    #[error("The email provided is already in use.")]
    EmailInUse,
    #[error("UserUseCase: {0}.")]
    Internal(String),
}