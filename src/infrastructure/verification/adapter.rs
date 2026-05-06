use anyhow::{Result, Context};

use crate::Config;

#[derive(Clone)]
pub struct RedisVerificationAdapter {
    pub conn: redis::aio::MultiplexedConnection,
}

impl RedisVerificationAdapter {
    pub async fn new(config: &Config) -> Result<Self> {
        let client = redis::Client::open(config.redis.url())
            .context("Failed to connect to Redis database.")?;

        let conn = client
            .get_multiplexed_async_connection()
            .await?;

        Ok(Self { conn })
    }
}