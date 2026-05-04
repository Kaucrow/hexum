use async_trait::async_trait;
use redis::AsyncCommands;
use thiserror::Error;

use crate::application::ports::output::{AuthRepository, AuthRepositoryError};
use super::RedisAdapter;

impl RedisAdapter {
    async fn do_store_session(&self, refresh_token: &str, user_id: &str, ttl_days: u64) -> Result<(), LocalError> {
        let mut conn = self.client
            .get_multiplexed_async_connection()
            .await?;

        let ttl_seconds = ttl_days * 24 * 60 * 60;

        // Saves the key and sets the expiration
        let _: () = conn.set_ex(refresh_token, user_id, ttl_seconds).await?;

        Ok(())
    }

    async fn do_consume_session(&self, refresh_token: &str) -> Result<Option<String>, LocalError> {
        let mut conn = self.client
            .get_multiplexed_async_connection()
            .await?;

        // Fetches the user_id and deletes the token
        let user_id: Option<String> = conn.get_del(refresh_token)
            .await
            .ok();

        Ok(user_id)
    }
}

#[async_trait]
impl AuthRepository for RedisAdapter {
    async fn store_session(&self, refresh_token: &str, user_id: &str, ttl_days: u64) -> Result<(), AuthRepositoryError> {
        Ok(self.do_store_session(refresh_token, user_id, ttl_days).await?)
    }

    async fn consume_session(&self, refresh_token: &str) -> Result<Option<String>, AuthRepositoryError> {
        Ok(self.do_consume_session(refresh_token).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
}

impl From<LocalError> for AuthRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Redis(e) => AuthRepositoryError::Internal(e.to_string()),
        }
    }
}