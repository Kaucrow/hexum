use anyhow::{Result, Context};
use lettre::{
    Tokio1Executor,
    transport::smtp::{
        AsyncSmtpTransport,
        authentication::Credentials,
    },
};

use crate::Config;

#[derive(Clone)]
pub struct LettreEmailAdapter {
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
    pub frontend_url: String,
}

impl LettreEmailAdapter {
    pub fn new(config: &Config) -> Result<Self> {
        let creds = Credentials::new(
            config.smtp.user.clone(),
            config.smtp.passwd.clone()
        );

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp.host)
            .context("Failed to create SMTP relay.")?
            .credentials(creds)
            .port(config.smtp.port)
            .build();

        Ok(Self {
            mailer,
            frontend_url: config.frontend.url(),
        })
    }
}