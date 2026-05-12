use std::sync::Arc;

use async_trait::async_trait;
use rand::distr::{Alphanumeric, SampleString};
use uuid::Uuid;

use crate::{
    application::ports::{
        input::{AuthTokens, AuthUseCase, AuthUseCaseError},
        output::{
            OAuthPort, OAuthPortError,
            SecurityPort, SecurityPortError,
            SessionPort, SessionPortError,
            UserRepository, UserRepositoryError,
        },
    }, domain::user::{AuthProvider, EmailAddress, User, UserAuthenticator}
};

#[derive(Clone)]
pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    session: Arc<dyn SessionPort>,
    security: Arc<dyn SecurityPort>,
    oauth: Arc<dyn OAuthPort>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        session: Arc<dyn SessionPort>,
        security: Arc<dyn SecurityPort>,
        oauth: Arc<dyn OAuthPort>,
    ) -> Self {
        Self { user_repo, session, security, oauth }
    }

    async fn resolve_and_login(
        &self,
        email_str: String,
        external_id: String,
        provider: AuthProvider,
    ) -> Result<AuthTokens, AuthUseCaseError> {
        let email = EmailAddress::new(email_str)
            .map_err(|e| AuthUseCaseError::Internal(e.to_string()))?;

        let user = match self.user_repo.get_user_by_email(&email).await {
            Some(existing_user) => {
                if !existing_user.is_active {
                    return Err(AuthUseCaseError::UserInactive);
                }
                // Link the provider if this is the user's first time using this OAuth provider for login
                self.ensure_provider_linked(&existing_user.id, provider, external_id).await?;
                existing_user
            }
            None => {
                // Completely new user
                self.register_oauth_user(email, provider, external_id).await?
            }
        };

        self.issue_session(&user.id).await
    }

    async fn ensure_provider_linked(
        &self,
        user_id: &Uuid,
        provider: AuthProvider,
        provider_id: String,
    ) -> Result<(), AuthUseCaseError> {
        let existing_auth = self.user_repo.get_authenticator(user_id, provider.clone()).await?;

        if existing_auth.is_none() {
            let new_auth = UserAuthenticator::new_oauth(*user_id, provider, provider_id);
            self.user_repo.add_authenticator(new_auth).await?;
        }

        Ok(())
    }

    async fn register_oauth_user(
        &self,
        email: EmailAddress,
        provider: AuthProvider,
        provider_id: String,
    ) -> Result<User, AuthUseCaseError> {
        let suffix = Alphanumeric.sample_string(&mut rand::rng(), 6);
        let temp_username = format!("user{}", suffix);

        let user = User::new(&temp_username, &email.as_str()).map_err(|e| AuthUseCaseError::Internal(e.to_string()))?;
        let auth = UserAuthenticator::new_oauth(user.id, provider, provider_id);

        self.user_repo.add_new_user(user.clone()).await?;
        self.user_repo.add_authenticator(auth).await?;

        Ok(user)
    }

    async fn issue_session(&self, user_id: &Uuid) -> Result<AuthTokens, AuthUseCaseError> {
        let access_token = self.security.generate_access_token(user_id)?;
        let refresh_token = self.security.generate_refresh_token();

        self.session
            .store_session(&refresh_token, user_id, 7)
            .await?;

        Ok(AuthTokens { access_token, refresh_token })
    }
}

#[async_trait]
impl AuthUseCase for AuthService {
    async fn login_user(&self, identity: &str, passwd: &str) -> Result<AuthTokens, AuthUseCaseError> {
        let user = if let Some(u) = self.user_repo.get_user_by_username(identity).await {
            u
        } else {
            // If the identity is not a username, try parsing is as email.
            // If it's not a valid email format, we stop here.
            let email = EmailAddress::new(identity.to_string())
                .or(Err(AuthUseCaseError::UserNotFound))?;

            self.user_repo.get_user_by_email(&email).await
                .ok_or(AuthUseCaseError::UserNotFound)?
        };

        if !user.is_active {
            return Err(AuthUseCaseError::UserInactive)
        }

        let local_authenticator = self.user_repo
            .get_authenticator(&user.id, AuthProvider::Local)
            .await?
            .ok_or(AuthUseCaseError::UserNotFound)?;

        let passwd_hash = local_authenticator.passwd
            .ok_or(AuthUseCaseError::Internal("User with local auth has no password set.".to_string()))?;

        if !self.security.verify_password(&passwd, &passwd_hash) {
            return Err(AuthUseCaseError::InvalidPassword);
        }

        let auth_tokens = self.issue_session(&user.id).await?;

        Ok(auth_tokens)
    }

    async fn login_user_via_google(&self, code: &str) -> Result<AuthTokens, AuthUseCaseError> {
        let google_user = self.oauth
            .get_google_user_info_by_code(code)
            .await
            .map_err(|e| AuthUseCaseError::Internal(format!("Google Auth failed: {:?}", e)))?;

        self.resolve_and_login(
            google_user.email,
            google_user.external_id,
            AuthProvider::Google,
        )
        .await
    }

    async fn login_user_via_github(&self, code: &str) -> Result<AuthTokens, AuthUseCaseError> {
        let github_user = self.oauth
            .get_github_user_info_by_code(code)
            .await
            .map_err(|e| AuthUseCaseError::Internal(format!("GitHub Auth failed: {:?}", e)))?;

        self.resolve_and_login(
            github_user.email,
            github_user.external_id.to_string(),
            AuthProvider::GitHub,
        )
        .await
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

        let auth_tokens = self.issue_session(&user_id).await?;

        Ok(auth_tokens)
    }

    async fn logout_user(&self, refresh_token: &str) -> Result<(), AuthUseCaseError> {
       self.session
            .consume_session(refresh_token)
            .await?
            .ok_or(AuthUseCaseError::InvalidRefreshToken)?;

        Ok(())
    }
}

impl From<UserRepositoryError> for AuthUseCaseError {
    fn from(e: UserRepositoryError) -> Self {
        match e {
            _ => AuthUseCaseError::Internal(e.to_string()),
        }
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

impl From<OAuthPortError> for AuthUseCaseError {
    fn from(e: OAuthPortError) -> Self {
        match e {
            OAuthPortError::InvalidCode => AuthUseCaseError::InvalidOAuthCode(e.to_string()),
            _ => AuthUseCaseError::Internal(e.to_string()),
        }
    }
}