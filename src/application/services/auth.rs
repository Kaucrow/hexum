use crate::{
    domain::user::User,
    application::ports::{
        input::{AuthUseCase, AuthTokens, AuthUseCaseError},
        output::{AuthRepository, UserRepository, SecurityPort},
    }
};
use async_trait::async_trait;

#[derive(Clone)]
pub struct AuthService<A, U, S>
where
    A: AuthRepository,
    U: UserRepository,
    S: SecurityPort,
{
    auth_repo: A,
    user_repo: U,
    security: S,
}

impl<A, U, S> AuthService<A, U, S>
where
    A: AuthRepository,
    U: UserRepository,
    S: SecurityPort,
{
    pub fn new(auth_repo: A, user_repo: U, security: S) -> Self {
        Self { auth_repo, user_repo, security }
    }
}

#[async_trait]
impl<A, U, S> AuthUseCase for AuthService<A, U, S>
where
    A: AuthRepository,
    U: UserRepository,
    S: SecurityPort,
{
    async fn login_user(&self, username: &str, password: &str) -> Result<AuthTokens, AuthUseCaseError> {
        let user = self.user_repo.get_user_by_username(username).await
            .ok_or(AuthUseCaseError::InvalidUsername)?;

        if !self.security.verify_password(&password, &user.password) {
            return Err(AuthUseCaseError::InvalidPassword);
        }

        let access_token = self.security.generate_access_token(&user.id).unwrap();
        let refresh_token = self.security.generate_refresh_token();

        self.auth_repo
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
        let user_id = self.auth_repo.consume_session(refresh_token)
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
        self.auth_repo.store_session(&refresh_token, &user.id, 7)
            .await
            .map_err(|e| AuthUseCaseError::Internal(e.to_string()))?;

        Ok(AuthTokens { access_token, refresh_token })
    }

    async fn logout_user(&self, refresh_token: &str) -> Result<(), AuthUseCaseError> {
        todo!();
    }
}