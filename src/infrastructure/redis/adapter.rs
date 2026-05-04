use crate::Config;
use anyhow::{Result, Context};

#[derive(Clone)]
pub struct RedisAdapter {
    pub client: redis::Client,
}

impl RedisAdapter {
    pub fn new(config: &Config) -> Result<Self> {
        let client = redis::Client::open(config.redis.url())
            .context("Failed to connect to Redis database.")?;

        Ok(Self { client })
    }
}