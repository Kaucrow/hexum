#[derive(Clone)]
pub struct GoogleAuthAdapter {
    pub client: reqwest::Client,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,   // Must match what the frontend sent
}

impl GoogleAuthAdapter {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            client_id,
            client_secret,
            redirect_uri,
        }
    }
}