use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::user::EmailAddress;

#[async_trait]
pub trait VerificationPort: Send + Sync + 'static {
    // Stores a token mapped to a User ID with an expiry
    async fn store_verification_token(&self, user_id: Uuid, token: &str, expires_in_secs: u64) -> Result<(), VerificationPortError>;
    // Retrieves user_id from token
    async fn consume_verification_token(&self, token: &str) -> Result<Uuid, VerificationPortError>;
}

#[derive(Error, Debug)]
pub enum VerificationPortError {
    #[error("{0}")]
    VerificationTokenInvalid(String),

    #[error("{0}")]
    Internal(String),
}

#[async_trait]
pub trait EmailPort: Send + Sync + 'static {
    async fn send_verification_email(&self, to: &EmailAddress, token: &str) -> Result<(), EmailPortError>;
}

#[derive(Error, Debug)]
pub enum EmailPortError {
    #[error("{0}")]
    Internal(String)
}