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

// Verify Argon2 Hash
pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

// Verify a PASETO v4 local access token & return the user ID
pub fn verify_access_token(token: &str, secret_key: &str) -> Result<String, pasetors::errors::Error> {
    let sk = SymmetricKey::<V4>::try_from(secret_key)?;

    let validation_rules = ClaimsValidationRules::new();
    let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)?;
    let trusted_token = local::decrypt(&sk, &untrusted_token, &validation_rules, None, None)?;

    let claims = trusted_token.payload_claims().unwrap().to_owned();

    let user_id: String = claims.get_claim("user_id")
    .and_then(|json_value| json_value.as_str())
    .map(|s| s.to_string())
    .ok_or(pasetors::errors::Error::Key)?;

    Ok(user_id)
}

pub fn generate_refresh_token() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 64)
}

// Generate a PASETO v4 local access token
pub fn generate_access_token(user_id: &str, secret_key: &str) -> Result<String, pasetors::errors::Error> {
    let mut claims = Claims::new()?;
    claims.add_additional("user_id", user_id)?;

    // Expiration will be 24 hours from current time
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("Timestamp should not overflow");

    // Format the expiration to RFC3339 and set it
    claims.expiration(&expiration.to_rfc3339())?;

    // Generate the key and encrypt the claims
    let sk = SymmetricKey::<V4>::try_from(secret_key)?;
    let token = local::encrypt(&sk, &claims, None, None)?;

    Ok(token)
}