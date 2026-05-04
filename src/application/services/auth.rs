use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    domain::user::User,
    application::ports::{
        input::{AuthUseCase, AuthTokens, AuthUseCaseError},
        output::{UserRepository, SessionPort, SecurityPort},
    }
};

#[derive(Clone)]
pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    session: Arc<dyn SessionPort>,
    security: Arc<dyn SecurityPort>,
}

impl AuthService {
    pub fn new(
        user_repo:Arc<dyn UserRepository>,
        session: Arc<dyn SessionPort>,
        security: Arc<dyn SecurityPort>
    ) -> Self {
        Self { user_repo, session, security }
    }
}

#[async_trait]
impl AuthUseCase for AuthService {
    async fn login_user(&self, username: &str, password: &str) -> Result<AuthTokens, AuthUseCaseError> {
        let user = self.user_repo.get_user_by_username(username).await
            .ok_or(AuthUseCaseError::InvalidUsername)?;

        if !self.security.verify_password(&password, &user.password) {
            return Err(AuthUseCaseError::InvalidPassword);
        }

        let access_token = self.security.generate_access_token(&user.id).unwrap();
        let refresh_token = self.security.generate_refresh_token();

        self.session
            .store_session(&refresh_token, &user.id, 7)
            .await
            .map_err(|e| AuthUseCaseError::Internal(e.to_string()))?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
        })
    }

    async fn verify_user(&self, access_token: &str) -> Result<User, AuthUseCaseError> {
        let user_id = self.security.verify_access_token(access_token).map_err(|e|
            AuthUseCaseError::InvalidAccessToken(e.to_string())
        )?;

        match self.user_repo.get_user_by_id(&user_id).await {
            Some(user) => Ok(user),
            None => Err(AuthUseCaseError::UserNotFound)
        }
    }

    async fn refresh_session(&self, refresh_token: &str) -> Result<AuthTokens, AuthUseCaseError> {
        // Consume the session
        let user_id = self.session
            .consume_session(refresh_token)
            .await
            .map_err(|e| AuthUseCaseError::Internal(e.to_string()))?
            .ok_or(AuthUseCaseError::InvalidRefreshToken)?;

        // Fetch user
        let user = self.user_repo.get_user_by_id(&user_id)
            .await
            .ok_or(AuthUseCaseError::UserNotFound)?;

        // Generate brand new tokens
        let access_token = self.security.generate_access_token(&user.id)
            .map_err(|e| AuthUseCaseError::Internal(e.to_string()))?;
        let refresh_token = self.security.generate_refresh_token();

        // Store the new session
        self.session
            .store_session(&refresh_token, &user.id, 7)
            .await
            .map_err(|e| AuthUseCaseError::Internal(e.to_string()))?;

        Ok(AuthTokens { access_token, refresh_token })
    }

    async fn logout_user(&self, refresh_token: &str) -> Result<(), AuthUseCaseError> {
        todo!();
    }
}