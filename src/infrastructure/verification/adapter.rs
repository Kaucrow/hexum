use anyhow::{Result, Context};

use crate::Config;

#[derive(Clone)]
pub struct RedisVerificationAdapter {
    pub conn: redis::aio::ConnectionManager,
}

impl RedisVerificationAdapter {
    pub async fn new(config: &Config) -> Result<Self> {
        let client = redis::Client::open(config.redis.url())?;

        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .context("Failed to connect to Redis database.")?;

        Ok(Self { conn })
    }
}