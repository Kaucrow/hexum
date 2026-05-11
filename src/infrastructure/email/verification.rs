use lettre::{
    Message,
    AsyncTransport,
    message::{SinglePart, MultiPart},
};
use async_trait::async_trait;
use thiserror::Error;
use askama::Template;

use crate::{
    domain::user::EmailAddress,
    application::ports::output::{EmailPort, EmailPortError},
};
use super::LettreEmailAdapter;

#[derive(Template)]
#[template(path = "verification_email.html")]
struct VerificationEmailTemplate<'a> {
    url: &'a str,
}

impl LettreEmailAdapter {
    async fn do_send_verification_email(&self, to: &EmailAddress, token: &str) -> Result<(), LocalError> {
        let url = format!("{}user/verify-ui?token={}", self.frontend_url, token);

        let template = VerificationEmailTemplate { url: &url };
        let html_body = template.render()?;

        let email = Message::builder()
            .from("No Reply <noreply@hexum.dev>".parse()?)
            .to(to.as_str().parse()?)
            .subject("Verify your account")
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::plain(format!("Verify your account here: {}", url))
                    )
                    .singlepart(
                        SinglePart::html(html_body)
                    )
            )?;

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
    #[error(transparent)]
    Askama(#[from] askama::Error),
}

impl From<LocalError> for EmailPortError {
    fn from(e: LocalError) -> Self {
        EmailPortError::Internal(e.to_string())
    }
}