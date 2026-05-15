use resend_rs::types::CreateEmailBaseOptions;
use async_trait::async_trait;
use thiserror::Error;
use askama::Template;

use crate::{
    domain::user::EmailAddress,
    application::ports::output::{EmailPort, EmailPortError},
};
use super::ResendEmailAdapter;

#[derive(Template)]
#[template(path = "verification_email.html")]
struct VerificationEmailTemplate<'a> {
    url: &'a str,
}

impl ResendEmailAdapter {
    async fn do_send_verification_email(&self, to: &EmailAddress, token: &str) -> Result<(), LocalError> {
        let url = format!("{}user/verify-ui?token={}", self.frontend_url, token);

        let template = VerificationEmailTemplate { url: &url };
        let html_body = template.render()?;

        let from = &self.from_addr;
        let to = [to.as_str()];
        let subject = "Verify your account";

        let email = CreateEmailBaseOptions::new(from, to, subject)
            .with_html(&html_body);

        self.client.emails.send(email).await?;

        Ok(())
    }
}

#[async_trait]
impl EmailPort for ResendEmailAdapter {
    async fn send_verification_email(&self, to: &EmailAddress, token: &str) -> Result<(), EmailPortError> {
        Ok(self.do_send_verification_email(to, token).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    Resend(#[from] resend_rs::Error),
    #[error(transparent)]
    Askama(#[from] askama::Error),
}

impl From<LocalError> for EmailPortError {
    fn from(e: LocalError) -> Self {
        EmailPortError::Internal(e.to_string())
    }
}