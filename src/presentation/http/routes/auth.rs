use std::sync::Arc;

use axum::{extract::State, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use time::Duration;

use crate::{
    prelude::*,
    AppState,
    application::{
        ports::input::{AuthUseCase, AuthUseCaseError},
    },
    presentation::http::{
        ApiError,
        dtos::auth::{LoginRequest, LoginResponse, LogoutResponse}
    },
};

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse, headers(
            ("Set-Cookie" = String, description = "HTTP-only cookies for access_token and refresh_token")
        )),
        (status = 401, description = "Unauthorized - Invalid username or password"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Authentication"]
)]
#[axum::debug_handler(state = AppState)]
pub async fn login(
    State(auth_service): State<Arc<dyn AuthUseCase>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), ApiError> {
    info!("Login attempt for user `{}`", &payload.username);

    let tokens = auth_service
        .login_user(&payload.username, &payload.password)
        .await?;

    // Attach cookies
    // Access token goes to the root path ("/")
    let access_cookie = build_cookie("access_token", tokens.access_token, "/");
    let refresh_cookie = build_cookie("refresh_token", tokens.refresh_token, "/auth/refresh-session");

    info!("Login successful for user `{}`", &payload.username);

    let response = LoginResponse { message: "Success".to_string() };
    Ok((jar.add(access_cookie).add(refresh_cookie), Json(response)))
}

#[utoipa::path(
    post,
    path = "/auth/refresh-session",
    responses(
        (status = 200, description = "Token refreshed successfully", headers(
            ("Set-Cookie" = String, description = "Updated HTTP-only cookies for access_token and refresh_token")
        )),
        (status = 401, description = "Unauthorized - Missing, invalid, or expired refresh token cookie"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Authentication"]
)]
#[axum::debug_handler(state = AppState)]
pub async fn refresh_session(
    State(auth_service): State<Arc<dyn AuthUseCase>>,
    jar: CookieJar,
) -> Result<CookieJar, ApiError> {
    info!("Session refresh requested");

    // Get the refresh token from the cookie
    let refresh_token = jar.get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(ApiError::Unauthorized("The refresh token is missing".to_string()))?;

    let new_tokens = auth_service
        .refresh_session(&refresh_token)
        .await?;

    // Return the updated cookies
    let access_cookie = build_cookie("access_token", new_tokens.access_token, "/");
    let refresh_cookie = build_cookie("refresh_token", new_tokens.refresh_token, "/auth/refresh-session");

    Ok(jar.add(access_cookie).add(refresh_cookie))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logout successful. Clears authentication cookies.", body=LogoutResponse, headers(
            ("Set-Cookie" = String, description = "Clears access_token and refresh_token cookies")
        )),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Authentication"]
)]
#[axum::debug_handler(state = AppState)]
pub async fn logout(
    State(auth_service): State<Arc<dyn AuthUseCase>>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<LogoutResponse>), ApiError> {
    info!("Logout requested");

    if let Some(cookie) = jar.get("refresh_token") {
        let _ = auth_service.logout_user(cookie.value()).await;
    }

    let access_cookie = build_removal_cookie("access_token", "/");
    let refresh_cookie = build_removal_cookie("refresh_token", "/auth/refresh-session");

    let response = LogoutResponse { message: "Logout successful".to_string() };
    Ok((jar.add(access_cookie).add(refresh_cookie), Json(response)))
}

// Helper function to build cookies
fn build_cookie<'a>(name: &'a str, value: String, path: &'a str) -> Cookie<'a> {
    Cookie::build((name, value))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path(path)
        .build()
}

// Helper function to build removal cookies
fn build_removal_cookie<'a>(name: &'a str, path: &'a str) -> Cookie<'a> {
    Cookie::build((name, ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path(path)
        .max_age(Duration::ZERO)
        .build()
}

impl From<AuthUseCaseError> for ApiError {
    fn from(e: AuthUseCaseError) -> Self {
        #[allow(unreachable_patterns)]
        match e {
            AuthUseCaseError::InvalidUsername => {
                warn!("Invalid username: {e}");
                ApiError::Unauthorized("Invalid username.".to_string())
            }
            AuthUseCaseError::InvalidPassword => {
                warn!("Invalid password: {e}");
                ApiError::Unauthorized("Invalid password.".to_string())
            }
            AuthUseCaseError::InvalidRefreshToken => {
                warn!("Invalid refresh token: {e}");
                ApiError::Unauthorized("The refresh token is invalid or expired.".to_string())
            }
            AuthUseCaseError::Internal(e) => {
                error!("An internal error occurred: {e}");
                ApiError::Internal("An internal error occurred.".to_string())
            }
            _ => {
                error!("Unexpected domain error: {e}");
                ApiError::Internal("An internal error occurred".to_string())
            }
        }
    }
}