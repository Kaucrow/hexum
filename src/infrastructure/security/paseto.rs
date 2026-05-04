use async_trait::async_trait;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use pasetors::{
    Local,
    claims::{Claims, ClaimsValidationRules},
    keys::SymmetricKey,
    version4::V4,
    token::UntrustedToken,
    local,
};
use chrono::{Utc, Duration};
use rand::distr::{Alphanumeric, SampleString};
use thiserror::Error;

use crate::application::ports::output::{SecurityPort, SecurityPortError};
use super::PasetoSecurityAdapter;

impl PasetoSecurityAdapter {
    fn do_verify_access_token(&self, token: &str) -> Result<String, LocalError> {
        let sk = SymmetricKey::<V4>::try_from(self.secret_key.as_str())
            .map_err(|_| LocalError::CryptoConfigError)?;

        let validation_rules = ClaimsValidationRules::new();
        let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
            .map_err(|_| LocalError::TokenVerificationFailed)?;
        let trusted_token = local::decrypt(&sk, &untrusted_token, &validation_rules, None, None)
            .map_err(|_| LocalError::TokenVerificationFailed)?;

        let user_id: String = trusted_token
            .payload_claims()
            .and_then(|claims| claims.get_claim("user_id"))
            .and_then(|json_value| json_value.as_str())
            .map(|s| s.to_string())
            .ok_or(LocalError::TokenVerificationFailed)?;

        Ok(user_id)
    }

    fn do_generate_access_token(&self, user_id: &str) -> Result<String, LocalError> {
        let mut claims = Claims::new()?;
        claims.add_additional("user_id", user_id)?;

        // Expiration will be 24 hours from current time
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("Timestamp should not overflow");

        // Format the expiration to RFC3339 and set it
        claims.expiration(&expiration.to_rfc3339())?;

        // Generate the key and encrypt the claims
        let sk = SymmetricKey::<V4>::try_from(self.secret_key.as_str())
            .map_err(|_| LocalError::CryptoConfigError)?;
        let token = local::encrypt(&sk, &claims, None, None)?;

        Ok(token)
    }
}

#[async_trait]
impl SecurityPort for PasetoSecurityAdapter {
    // Verify Argon2 hash
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(hash) => hash,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    // Verify a PASETO v4 access token & return the user_id
    fn verify_access_token(&self, token: &str) -> Result<String, SecurityPortError> {
        Ok(self.do_verify_access_token(token)?)
    }

    // Generate a PASETO v4 access token
    fn generate_access_token(&self, user_id: &str) -> Result<String, SecurityPortError> {
        Ok(self.do_generate_access_token(user_id)?)
    }

    // Generate a 64-characters long refresh token
    fn generate_refresh_token(&self) -> String {
        Alphanumeric.sample_string(&mut rand::rng(), 64)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("")]
    TokenVerificationFailed,
    #[error("")]
    CryptoConfigError,
    #[error(transparent)]
    Paseto(#[from] pasetors::errors::Error),
}

impl From<LocalError> for SecurityPortError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::TokenVerificationFailed => SecurityPortError::TokenVerificationFailed,
            LocalError::CryptoConfigError => SecurityPortError::Internal("Invalid Secret Key provided.".to_string()),
            LocalError::Paseto(e) => SecurityPortError::Internal(e.to_string()),
        }
    }
}