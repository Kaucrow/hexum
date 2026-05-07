use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait GoogleAuthPort: Send + Sync {
    /// Exchanges a code for user info (email, name, google_id)
    async fn get_user_info_by_code(&self, code: &str) -> Result<GoogleUserInfo, GoogleAuthPortError>;
}

pub struct GoogleUserInfo {
    pub email: String,
    pub external_id: String,
}

#[derive(Debug, Error)]
pub enum GoogleAuthPortError {
    #[error("The authorization code provided is invalid or has expired")]
    InvalidCode,

    #[error("A network error occurred while communicating with Google: {0}")]
    NetworkError(String),

    #[error("Failed to parse user info response from Google")]
    ParseError,

    #[error("GoogleAuth: {0}")]
    Internal(String),
}