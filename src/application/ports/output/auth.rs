use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[async_trait]
pub trait SessionPort: Send + Sync + 'static {
    // Stores the refresh token and associates it with a user ID for a given number of days
    async fn store_session(&self, refresh_token: &str, user_id: &Uuid, ttl_days: u64) -> Result<(), SessionPortError>;
    // Fetches the user ID associated with the token and deletes the token
    async fn consume_session(&self, refresh_token: &str) -> Result<Option<Uuid>, SessionPortError>;
}

#[derive(Error, Debug)]
pub enum SessionPortError {
    #[error("{0}")]
    Internal(String)
}

#[async_trait]
pub trait SecurityPort: Send + Sync + 'static {
    fn verify_password(&self, password: &str, hash: &str) -> bool;
    fn hash(&self, s: &str) -> Result<String, SecurityPortError>;
    fn verify_access_token(&self, token: &str) -> Result<Uuid, SecurityPortError>;
    fn generate_access_token(&self, user_id: &Uuid) -> Result<String, SecurityPortError>;
    fn generate_refresh_token(&self) -> String;
    fn generate_verification_token(&self) -> String;
}

#[derive(Error, Debug)]
pub enum SecurityPortError {
    #[error("The token provided is invalid or expired.")]
    TokenVerificationFailed,

    #[error("Security: {0}")]
    Internal(String),
}