use async_trait::async_trait;
use redis::AsyncCommands;
use thiserror::Error;
use uuid::Uuid;

use crate::application::ports::output::{VerificationPort, VerificationPortError};
use super::RedisVerificationAdapter;

impl RedisVerificationAdapter {
    async fn do_store_verification_token(&self, user_id: Uuid, token: &str, expires_in_secs: u64) -> Result<(), LocalError> {
        let key = self.format_key(token);

        let _: () = self.conn.clone().set_ex(key, user_id.to_string(), expires_in_secs)
            .await?;
        Ok(())
    }

    async fn do_consume_verification_token(&self, token: &str) -> Result<Uuid, LocalError> {
        let key = self.format_key(token);

        let user_id: String = self.conn.clone().get_del::<&str, Option<String>>(&key)
            .await?
            .ok_or(LocalError::VerificationTokenInvalid)?;

        let user_id_uuid = Uuid::try_parse(&user_id)?;

        Ok(user_id_uuid)
    }

    fn format_key(&self, token: &str) -> String {
        format!("verify:{token}")
    }
}

#[async_trait]
impl VerificationPort for RedisVerificationAdapter {
    async fn store_verification_token(&self, user_id: Uuid, token: &str, expires_in_secs: u64) -> Result<(), VerificationPortError> {
        Ok(self.do_store_verification_token(user_id, token, expires_in_secs).await?)
    }

    async fn consume_verification_token(&self, token: &str) -> Result<Uuid, VerificationPortError> {
        Ok(self.do_consume_verification_token(token).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("The verification token is invalid or expired.")]
    VerificationTokenInvalid,
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
}

impl From<LocalError> for VerificationPortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::VerificationTokenInvalid => VerificationPortError::VerificationTokenInvalid(e.to_string()),
            LocalError::Redis(e) => VerificationPortError::Internal(e.to_string()),
            LocalError::Uuid(e) => VerificationPortError::Internal(e.to_string()),
        }
    }
}