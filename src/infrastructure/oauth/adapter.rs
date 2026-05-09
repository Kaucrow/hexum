use crate::Config;

#[derive(Clone)]
pub struct OAuthAdapter {
    pub client: reqwest::Client,
    pub redirect_uri: String,   // Must match what the frontend sent
    pub google: OAuthData,
    pub github: OAuthData,
}

#[derive(Clone)]
pub struct OAuthData {
    pub client_id: String,
    pub client_secret: String,
}

impl OAuthAdapter {
    pub fn new(config: &Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            redirect_uri: config.oauth.redirect_url(config.frontend.url()),
            google: OAuthData {
                client_id: config.oauth.google.client_id.clone(),
                client_secret: config.oauth.google.client_secret.clone(),
            },
            github: OAuthData {
                client_id: config.oauth.github.client_id.clone(),
                client_secret: config.oauth.github.client_secret.clone(),
            },
        }
    }
}