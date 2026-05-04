use crate::prelude::*;
use strum::Display;
use local_ip_address::local_ip;
use rand::distr::{Alphanumeric, SampleString};

#[derive(Deserialize, Clone)]
pub struct Config {
    pub debug: bool,
    pub api: ApiConfig,
    #[serde(rename = "postgresql")]
    pub postgres: PostgresConfig,
    pub redis: RedisConfig,
    #[serde(default)]
    pub session: SessionConfig,
}

#[derive(Deserialize, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub protocol: ApiProtocol,
    #[serde(default)]
    pub domain: String,
    pub docs_endpoint: String,
}

impl ApiConfig {
    pub fn url(&self) -> String {
        match self.protocol {
            ApiProtocol::Https => format!("https://{}", self.domain),
            ApiProtocol::Http => format!("http://{}:{}", self.domain, self.port),
        }
    }
}

#[derive(Deserialize, Clone, Debug, Display, Default)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ApiProtocol {
    #[default]
    Http,
    Https,
}

#[derive(Deserialize, Clone)]
pub struct PostgresConfig {
    pub pool_max_conn: u32,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub passwd: String,
    pub name: String,
}

impl PostgresConfig {
    pub fn url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user, self.passwd, self.host, self.port, self.name
        )
    }
}

#[derive(Deserialize, Clone)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub passwd: String,
    pub number: u32,
}

impl RedisConfig {
    pub fn url(&self) -> String {
        format!(
            "redis://{}:{}@{}:{}/{}",
            self.user, self.passwd, self.host, self.port, self.number
        )
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct SessionConfig {
    #[serde(default)]
    pub secret_key: String,
}

pub fn get_config() -> Result<Config, config::ConfigError> {
    let base_path = get_base_path();

    let environment: String = std::env::var("RAILWAY_ENVIRONMENT_NAME")
        .unwrap_or_else(|_| "development".into());

    let config_directory = base_path.join(format!("config/{}", environment));

    let filename = "base.toml";

    let settings = config::Config::builder()
        .add_source(config::File::from(
            config_directory.join(filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    let mut app_config = settings.try_deserialize::<Config>()?;

    if environment == "production" {
        app_config.api.protocol = ApiProtocol::Https;
        app_config.api.domain = std::env::var("RAILWAY_PUBLIC_DOMAIN").expect("Failed to get Railway public domain.");
    } else {
        let local_ip = local_ip().unwrap_or("127.0.0.1".parse().unwrap()).to_string();

        app_config.api.protocol = ApiProtocol::Http;
        app_config.api.domain = local_ip.clone();
    }

    app_config.session.secret_key = Alphanumeric.sample_string(&mut rand::rng(), 32);

    Ok(app_config)
}