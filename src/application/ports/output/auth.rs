use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait SessionPort: Send + Sync + 'static {
    // Stores the refresh token and associates it with a user ID for a given number of days
    async fn store_session(&self, refresh_token: &str, user_id: &str, ttl_days: u64) -> Result<(), SessionPortError>;
    // Fetches the user ID associated with the token and deletes the token
    async fn consume_session(&self, refresh_token: &str) -> Result<Option<String>, SessionPortError>;
}

#[derive(Error, Debug)]
pub enum SessionPortError {
    #[error("Internal error: {0}.")]
    Internal(String)
}

#[async_trait]
pub trait SecurityPort: Send + Sync + 'static {
    fn verify_password(&self, password: &str, hash: &str) -> bool;
    fn verify_access_token(&self, token: &str) -> Result<String, SecurityPortError>;
    fn generate_access_token(&self, user_id: &str) -> Result<String, SecurityPortError>;
    fn generate_refresh_token(&self) -> String;
}

#[derive(Error, Debug)]
pub enum SecurityPortError {
    #[error("The token provided is invalid or expired.")]
    TokenVerificationFailed,

    #[error("Internal error: {0}.")]
    Internal(String),
}