use anyhow::Result;
use lettre::{
    Tokio1Executor,
    transport::smtp::AsyncSmtpTransport
};

#[derive(Clone)]
pub struct LettreEmailAdapter {
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl LettreEmailAdapter {
    pub fn new() -> Result<Self> {
        let mailer = AsyncSmtpTransport::unencrypted_localhost();

        Ok(Self { mailer })
    }
}