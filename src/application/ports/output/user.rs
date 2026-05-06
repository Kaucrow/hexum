use async_trait::async_trait;
use thiserror::Error;

use crate::domain::user::User;

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn get_user_by_id(&self, id: &str) -> Option<User>;
    async fn get_user_by_username(&self, username: &str) -> Option<User>;
    async fn add_new_user(&self, user: User) -> Result<(), UserRepositoryError>;
}

#[derive(Error, Debug)]
pub enum UserRepositoryError {
    #[error("The username provided is already in use.")]
    UsernameInUse,
    #[error("The email provided is already in use.")]
    EmailInUse,
    #[error("{0}")]
    Internal(String),
}