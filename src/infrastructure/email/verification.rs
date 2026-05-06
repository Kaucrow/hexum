use async_trait::async_trait;
use thiserror::Error;
use lettre::{Message, AsyncTransport};

use crate::{
    domain::user::EmailAddress,
    application::ports::output::{EmailPort, EmailPortError},
};
use super::LettreEmailAdapter;

impl LettreEmailAdapter {
    async fn do_send_verification_email(&self, to: &EmailAddress, token: &str) -> Result<(), LocalError> {
        let url = format!("https://hexum.dev/verify?token={}", token);
        let email = Message::builder()
            .from("noreply@hexum.dev".parse()?)
            .to(to.as_str().parse()?)
            .subject("Verify your account")
            .body(format!("Click here to verify: {}", url))?;

        self.mailer.send(email).await?;

        Ok(())
    }
}

#[async_trait]
impl EmailPort for LettreEmailAdapter {
    async fn send_verification_email(&self, to: &EmailAddress, token: &str) -> Result<(), EmailPortError> {
        Ok(self.do_send_verification_email(to, token).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    LettreError(#[from] lettre::error::Error),
    #[error(transparent)]
    LettreSmtp(#[from] lettre::transport::smtp::Error),
    #[error(transparent)]
    LettreAddress(#[from] lettre::address::AddressError),
}

impl From<LocalError> for EmailPortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::LettreError(e) => EmailPortError::Internal(e.to_string()),
            LocalError::LettreSmtp(e) => EmailPortError::Internal(e.to_string()),
            LocalError::LettreAddress(e) => EmailPortError::Internal(e.to_string()),
        }
    }
}