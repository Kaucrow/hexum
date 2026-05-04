use crate::Config;
use super::queries;
use sqlx::{
    PgPool,
    postgres::PgPoolOptions,
};
use anyhow::{Result, Context};

#[derive(Clone)]
pub struct PostgresAdapter {
    pub pool: PgPool,
}

impl PostgresAdapter {
    pub async fn new(config: &Config) -> Result<Self> {
        queries::init()?;

        let pool = PgPoolOptions::new()
            .max_connections(config.postgres.pool_max_conn)
            .connect(&config.postgres.url())
            .await
            .context("Failed to connect to PostgreSQL database.")?;

        Ok(Self { pool })
    }
}