use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    domain::user::{User, EmailAddress},
    application::ports::{
        input::{AuthUseCase, AuthTokens, AuthUseCaseError},
        output::{
            UserRepository,
            SessionPort, SessionPortError,
            SecurityPort, SecurityPortError,
            GoogleAuthPort, GoogleAuthPortError,
        },
    }
};

#[derive(Clone)]
pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    session: Arc<dyn SessionPort>,
    security: Arc<dyn SecurityPort>,
    google: Arc<dyn GoogleAuthPort>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        session: Arc<dyn SessionPort>,
        security: Arc<dyn SecurityPort>,
        google: Arc<dyn GoogleAuthPort>,
    ) -> Self {
        Self { user_repo, session, security, google }
    }
}

#[async_trait]
impl AuthUseCase for AuthService {
    async fn login_user(&self, username: &str, password: &str) -> Result<AuthTokens, AuthUseCaseError> {
        let user = self.user_repo.get_user_by_username(username).await
            .ok_or(AuthUseCaseError::InvalidUsername)?;

        if !user.is_active {
            return Err(AuthUseCaseError::UserInactive)
        }

        if !self.security.verify_password(&password, &user.passwd) {
            return Err(AuthUseCaseError::InvalidPassword);
        }

        let access_token = self.security.generate_access_token(&user.id).unwrap();
        let refresh_token = self.security.generate_refresh_token();

        self.session
            .store_session(&refresh_token, &user.id, 7)
            .await?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
        })
    }

    async fn login_user_via_google(&self, code: &str) -> Result<AuthTokens, AuthUseCaseError> {
        let google_user = self.google
            .get_user_info_by_code(code)
            .await
            .map_err(|e| AuthUseCaseError::Internal(format!("Google Auth failed: {:?}", e)))?;

        let google_user_email = EmailAddress::new(google_user.email)
            .map_err(|e| AuthUseCaseError::Internal(format!("Tried to parse an invalid email: {:?}", e)))?;

        let user = self.user_repo
            .get_user_by_email(&google_user_email)
            .await
            .ok_or(AuthUseCaseError::UserNotFound)?;

        if !user.is_active {
            return Err(AuthUseCaseError::UserInactive);
        }

        let access_token = self.security.generate_access_token(&user.id)?;

        let refresh_token = self.security.generate_refresh_token();

        self.session
            .store_session(&refresh_token, &user.id, 7)
            .await?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
        })
    }

    async fn verify_user(&self, access_token: &str) -> Result<User, AuthUseCaseError> {
        let user_id = self.security.verify_access_token(access_token)?;

        match self.user_repo.get_user_by_id(&user_id).await {
            Some(user) => {
                if !user.is_active {
                    return Err(AuthUseCaseError::UserInactive)
                }

                Ok(user)
            },
            None => Err(AuthUseCaseError::UserNotFound)
        }
    }

    async fn refresh_session(&self, refresh_token: &str) -> Result<AuthTokens, AuthUseCaseError> {
        // Consume the session
        let user_id = self.session
            .consume_session(refresh_token)
            .await?
            .ok_or(AuthUseCaseError::InvalidRefreshToken)?;

        // Fetch user
        let user = self.user_repo.get_user_by_id(&user_id)
            .await
            .ok_or(AuthUseCaseError::UserNotFound)?;

        if !user.is_active {
            return Err(AuthUseCaseError::UserInactive)
        }

        // Generate brand new tokens
        let access_token = self.security.generate_access_token(&user.id)?;
        let refresh_token = self.security.generate_refresh_token();

        // Store the new session
        self.session
            .store_session(&refresh_token, &user.id, 7)
            .await?;

        Ok(AuthTokens { access_token, refresh_token })
    }

    async fn logout_user(&self, refresh_token: &str) -> Result<(), AuthUseCaseError> {
       self.session
            .consume_session(refresh_token)
            .await?
            .ok_or(AuthUseCaseError::InvalidRefreshToken)?;

        Ok(())
    }
}

impl From<SessionPortError> for AuthUseCaseError {
    fn from(e: SessionPortError) -> Self {
        match e {
            _ => AuthUseCaseError::Internal(e.to_string()),
        }
    }
}

impl From<SecurityPortError> for AuthUseCaseError {
    fn from(e: SecurityPortError) -> Self {
        match e {
            SecurityPortError::TokenVerificationFailed => AuthUseCaseError::InvalidAccessToken(e.to_string()),
            _ => AuthUseCaseError::Internal(e.to_string()),
        }
    }
}

impl From<GoogleAuthPortError> for AuthUseCaseError {
    fn from(e: GoogleAuthPortError) -> Self {
        match e {
            GoogleAuthPortError::InvalidCode => AuthUseCaseError::InvalidOAuthCode(e.to_string()),
            _ => AuthUseCaseError::Internal(e.to_string()),
        }
    }
}