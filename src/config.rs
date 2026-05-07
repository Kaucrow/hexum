use crate::prelude::*;
use strum::{Display, EnumString};
use local_ip_address::local_ip;

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub environment: Environment,
    pub api: ApiConfig,
    pub frontend: FrontendConfig,
    #[serde(rename = "postgresql")]
    pub postgres: PostgresConfig,
    pub redis: RedisConfig,
    #[serde(default)]
    pub session: SessionConfig,
    pub smtp: SmtpConfig,
    pub oauth: OAuthConfig,
}

#[derive(Deserialize, Clone, Debug, Display, Default, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Development,
    Production,
}

#[derive(Deserialize, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub domain: String,
    pub port: u16,
    #[serde(default)]
    pub protocol: ApiProtocol,
    pub docs_endpoint: String,
}

impl ApiConfig {
    pub fn url(&self) -> String {
        match self.protocol {
            ApiProtocol::Http => format!("http://{}:{}/", self.domain, self.port),
            ApiProtocol::Https => format!("https://{}/", self.domain),
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
pub struct FrontendConfig {
    pub domain: String,
    pub port: u16,
    #[serde(default)]
    pub protocol: FrontendProtocol,
}

impl FrontendConfig {
    pub fn url(&self) -> String {
        match self.protocol {
            FrontendProtocol::Http => format!("http://{}:{}/", self.domain, self.port),
            FrontendProtocol::Https => format!("https://{}/", self.domain),
            FrontendProtocol::Hexum => format!("hexum://",)
        }
    }
}

#[derive(Deserialize, Clone, Debug, Display, Default)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum FrontendProtocol {
    #[default]
    Http,
    Https,
    Hexum
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
        if self.passwd.is_empty() {
            format!("redis://{}:{}/{}", self.host, self.port, self.number)
        } else {
            format!("redis://{}:{}@{}:{}/{}",
                self.user, self.passwd,
                self.host, self.port, self.number)
        }
    }
}

#[derive(Deserialize, Clone, Default)]
pub struct SessionConfig {}

#[derive(Deserialize, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub passwd: String,
}

#[derive(Deserialize, Clone)]
pub struct OAuthConfig {
    pub login_ui_endpoint: String,
    pub callback_endpoint: String,
    pub google: GoogleConfig,
}

impl OAuthConfig {
    pub fn login_ui_url(&self, frontend_url: String) -> String {
        format!("{}{}", frontend_url, self.login_ui_endpoint)
    }

    pub fn redirect_url(&self, frontend_url: String) -> String {
        format!("{}{}", frontend_url, self.callback_endpoint)
    }
}

#[derive(Deserialize, Clone)]
pub struct GoogleConfig {
    pub login_endpoint: String,
    pub client_id: String,
    pub client_secret: String,
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

    app_config.environment = environment
        .to_lowercase()
        .parse()
        .unwrap_or_default();

    if environment == "production" {
        app_config.api.protocol = ApiProtocol::Https;
        app_config.api.domain = std::env::var("RAILWAY_PUBLIC_DOMAIN").expect("Failed to get Railway public domain.");
    } else {
        let local_ip = local_ip().unwrap_or("127.0.0.1".parse().unwrap()).to_string();

        app_config.api.protocol = ApiProtocol::Http;
        app_config.api.domain = local_ip.clone();
    }

    Ok(app_config)
}