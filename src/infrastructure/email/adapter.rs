use anyhow::Context;
use lettre::{
    Tokio1Executor,
    transport::smtp::{
        AsyncSmtpTransport,
        authentication::Credentials,
    },
};
use resend_rs::Resend;

use crate::Config;

#[derive(Clone)]
pub struct LettreEmailAdapter {
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
    pub frontend_url: String,
    pub from_addr: String,
}

impl LettreEmailAdapter {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let smtp_config = config.email.sender.smtp_config()?;

        let creds = Credentials::new(
            smtp_config.user.clone(),
            smtp_config.passwd.clone()
        );

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_config.host)
            .context("Failed to create SMTP relay.")?
            .credentials(creds)
            .port(smtp_config.port)
            .build();

        Ok(Self {
            mailer,
            frontend_url: config.frontend.url(),
            from_addr: config.email.from.clone(),
        })
    }
}

#[derive(Clone)]
pub struct ResendEmailAdapter {
    pub client: Resend,
    pub frontend_url: String,
    pub from_addr: String,
}

impl ResendEmailAdapter {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let resend_config = config.email.sender.resend_config()?;

        let client = Resend::new(&resend_config.api_key);

        Ok(Self {
            client,
            frontend_url: config.frontend.url(),
            from_addr: config.email.from.clone(),
        })
    }
}