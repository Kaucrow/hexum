use crate::Config;
use anyhow::{Result, Context};

#[derive(Clone)]
pub struct RedisSessionAdapter {
    pub client: redis::Client,
}

impl RedisSessionAdapter {
    pub fn new(config: &Config) -> Result<Self> {
        let client = redis::Client::open(config.redis.url())
            .context("Failed to connect to Redis database.")?;

        Ok(Self { client })
    }
}