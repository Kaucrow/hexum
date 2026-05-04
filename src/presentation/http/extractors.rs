use crate::{
    Config,
    AppState,
    domain::user::User,
    application::ports::UserRepository,
    application::auth::verify_access_token,
};

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::cookie::CookieJar;
use std::sync::Arc;

pub struct AuthenticatedUser(pub User);

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {

        // Pull dependencies from AppState
        let user_repo: Arc<dyn UserRepository> = axum::extract::FromRef::from_ref(state);
        let config: Arc<Config> = axum::extract::FromRef::from_ref(state);

        // Grab the CookieJar from the incoming headers
        let jar = CookieJar::from_headers(&parts.headers);

        // Extract the "access_token" cookie
        let access_token = jar.get("access_token")
            .map(|cookie| cookie.value())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Verify the PASETO token & get the user ID.
        // This fails if the token is tampered with or the token expired.
        let user_id = verify_access_token(access_token, &config.session.secret_key)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        // Get the user by their ID
        match user_repo.get_user_by_id(&user_id).await {
            Some(user) => Ok(AuthenticatedUser(user)),
            None => Err(StatusCode::UNAUTHORIZED),  // User might have been deleted
        }
    }
}