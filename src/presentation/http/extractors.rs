use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    prelude::*,
    AppState,
    domain::user::User,
    application::ports::input::AuthUseCase,
};

pub struct AuthenticatedUser(pub User);

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        info!("Verification request received");

        // Pull dependencies from AppState
        let auth_service: Arc<dyn AuthUseCase> = axum::extract::FromRef::from_ref(state);

        // Grab the CookieJar from the incoming headers
        let jar = CookieJar::from_headers(&parts.headers);

        // Extract the "access_token" cookie
        let access_token = jar.get("access_token")
            .map(|cookie| cookie.value())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let user = auth_service
            .verify_user(access_token)
            .await
            .map_err(|e| {
                warn!("Verification failed: {e}");
                StatusCode::UNAUTHORIZED
            })?;

        Ok(Self(user))
    }
}