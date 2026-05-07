use async_trait::async_trait;
use thiserror::Error;

use crate::domain::user::User;

#[async_trait]
pub trait AuthUseCase: Send + Sync + 'static {
    async fn login_user(&self, username: &str, password: &str) -> Result<AuthTokens, AuthUseCaseError>;
    async fn login_user_via_google(&self, code: &str) -> Result<AuthTokens, AuthUseCaseError>;
    async fn verify_user(&self, access_token: &str) -> Result<User, AuthUseCaseError>;
    async fn refresh_session(&self, refresh_token: &str) -> Result<AuthTokens, AuthUseCaseError>;
    async fn logout_user(&self, refresh_token: &str) -> Result<(), AuthUseCaseError>;
}

pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Error, Debug)]
pub enum AuthUseCaseError {
    #[error("The username provided is invalid.")]
    InvalidUsername,

    #[error("The password provided is invalid.")]
    InvalidPassword,

    #[error("The access token provided is invalid: {0}")]
    InvalidAccessToken(String),

    #[error("The refresh token provided is invalid")]
    InvalidRefreshToken,

    #[error("{0}")]
    InvalidOAuthCode(String),

    #[error("The user could not be found.")]
    UserNotFound,

    #[error("The user is inactive.")]
    UserInactive,

    #[error("AuthUseCase: {0}")]
    Internal(String),
}