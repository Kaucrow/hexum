use async_trait::async_trait;

use crate::{
    prelude::*,
    application::ports::output::{GoogleAuthPort, GoogleAuthPortError, GoogleUserInfo},
};
use super::GoogleAuthAdapter;

#[async_trait]
impl GoogleAuthPort for GoogleAuthAdapter {
    async fn get_user_info_by_code(&self, code: &str) -> Result<GoogleUserInfo, GoogleAuthPortError> {
        // Exchange code for access_token
        let token_params = [
            ("code", code),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("redirect_uri", &self.redirect_uri),
            ("grant_type", "authorization_code"),
        ];

        let token_res = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&token_params)
            .send()
            .await
            .map_err(|e| GoogleAuthPortError::NetworkError(e.to_string()))?;

        if !token_res.status().is_success() {
            return Err(GoogleAuthPortError::InvalidCode);
        }

        let token_data: GoogleTokenResponse = token_res
            .json()
            .await
            .map_err(|_| GoogleAuthPortError::ParseError)?;

        // Get user info using the token
        let user_res = self.client
            .get("https://www.googleapis.com/oauth2/v3/userinfo")
            .bearer_auth(token_data.access_token)
            .send()
            .await
            .map_err(|e| GoogleAuthPortError::NetworkError(e.to_string()))?;

        let user_data: GoogleUserResponse = user_res
            .json()
            .await
            .map_err(|_| GoogleAuthPortError::ParseError)?;

        Ok(GoogleUserInfo {
            email: user_data.email,
            external_id: user_data.sub,
        })
    }
}

#[derive(Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GoogleUserResponse {
    sub: String,    // Google's unique user ID
    email: String,
}