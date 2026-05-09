use async_trait::async_trait;

use crate::{
    prelude::*,
    application::ports::output::{OAuthPort, OAuthPortError, GoogleUserInfo, GitHubUserInfo},
};
use super::OAuthAdapter;

#[async_trait]
impl OAuthPort for OAuthAdapter {
    async fn get_google_user_info_by_code(&self, code: &str) -> Result<GoogleUserInfo, OAuthPortError> {
        // Exchange code for access_token
        let token_params = [
            ("code", code),
            ("client_id", &self.google.client_id),
            ("client_secret", &self.google.client_secret),
            ("redirect_uri", &self.redirect_uri),
            ("grant_type", "authorization_code"),
        ];

        let token_res = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&token_params)
            .send()
            .await
            .map_err(|e| OAuthPortError::NetworkError(e.to_string()))?;

        if !token_res.status().is_success() {
            return Err(OAuthPortError::InvalidCode);
        }

        let token_data: GoogleTokenResponse = token_res
            .json()
            .await
            .map_err(|_| OAuthPortError::ParseError)?;

        // Get user info using the token
        let user_res = self.client
            .get("https://www.googleapis.com/oauth2/v3/userinfo")
            .bearer_auth(token_data.access_token)
            .send()
            .await
            .map_err(|e| OAuthPortError::NetworkError(e.to_string()))?;

        let user_data: GoogleUserResponse = user_res
            .json()
            .await
            .map_err(|_| OAuthPortError::ParseError)?;

        Ok(GoogleUserInfo {
            email: user_data.email,
            external_id: user_data.sub,
        })
    }

    async fn get_github_user_info_by_code(&self, code: &str) -> Result<GitHubUserInfo, OAuthPortError> {
        // Exchange code for access_token
        let token_params = [
            ("code", code),
            ("client_id", &self.github.client_id),
            ("client_secret", &self.github.client_secret),
            ("redirect_uri", &self.redirect_uri),
        ];

        let token_res = self.client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&token_params)
            .send()
            .await
            .map_err(|e| OAuthPortError::NetworkError(e.to_string()))?;

        let token_data: GitHubTokenResponse = token_res
            .json()
            .await
            .map_err(|_| OAuthPortError::ParseError)?;

        // Get user info using the token
        let user_res = self.client
            .get("https://api.github.com/user")
            .header("User-Agent", "hexum")
            .bearer_auth(token_data.access_token)
            .send()
            .await
            .map_err(|e| OAuthPortError::NetworkError(e.to_string()))?;

        let user_data: GitHubUserResponse = user_res
            .json()
            .await
            .map_err(|_| OAuthPortError::ParseError)?;

        let final_email = user_data.email.unwrap_or(format!("{}@users.noreply.github.com", user_data.login));

        Ok(GitHubUserInfo {
            email: final_email,
            external_id: user_data.id,
            username: user_data.login,
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

#[derive(Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GitHubUserResponse {
    id: i64,        // GitHub's unique numeric ID
    login: String,  // The username
    email: Option<String>,
}