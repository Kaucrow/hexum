use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    domain::user::User,
    application::ports::{
        input::{UserUseCase, UserUseCaseError},
        output::{
            UserRepository, UserRepositoryError,
            VerificationPort, VerificationPortError,
            SecurityPort,
            EmailPort, EmailPortError,
        },
    }
};

#[derive(Clone)]
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    verification: Arc<dyn VerificationPort>,
    security: Arc<dyn SecurityPort>,
    email: Arc<dyn EmailPort>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        verification: Arc<dyn VerificationPort>,
        security: Arc<dyn SecurityPort>,
        email: Arc<dyn EmailPort>,
    ) -> Self {
        Self { user_repo, verification, security, email }
    }
}

#[async_trait]
impl UserUseCase for UserService {
    async fn register_user(&self, user: User) -> Result<(), UserUseCaseError> {
        let user_id = user.id.clone();
        let user_email = user.email.clone();

        self.user_repo.add_new_user(user).await?;

        let verification_token = self.security.generate_verification_token();

        self.verification.store_verification_token(user_id, &verification_token, 1800).await?;

        self.email.send_verification_email(&user_email, &verification_token).await?;

        Ok(())
    }
}

impl From<UserRepositoryError> for UserUseCaseError {
    fn from(e: UserRepositoryError) -> Self {
        match e {
            UserRepositoryError::UsernameInUse => UserUseCaseError::UsernameInUse,
            UserRepositoryError::EmailInUse => UserUseCaseError::EmailInUse,
            _ => UserUseCaseError::Internal(e.to_string()),
        }
    }
}

impl From<VerificationPortError> for UserUseCaseError {
    fn from(e: VerificationPortError) -> Self {
        match e {
            _ => UserUseCaseError::Internal(e.to_string()),
        }
    }
}

impl From<EmailPortError> for UserUseCaseError {
    fn from(e: EmailPortError) -> Self {
        match e {
            _ => UserUseCaseError::Internal(e.to_string()),
        }
    }
}