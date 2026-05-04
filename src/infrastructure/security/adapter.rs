#[derive(Clone)]
pub struct PasetoSecurityAdapter {
    pub secret_key: String,
}

impl PasetoSecurityAdapter {
    pub fn new(secret_key: String) -> Self {
        Self { secret_key }
    }
}