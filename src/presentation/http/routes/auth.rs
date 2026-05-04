use crate::{
    prelude::*,
    AppState,
    application::{
        ports::input::{AuthUseCase, AuthUseCaseError},
    },
    presentation::http::dtos::auth::{LoginRequest, LoginResponse},
};
use axum::{extract::State, Json, http::StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Unauthorized - Invalid credentials"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Authentication"]
)]
#[axum::debug_handler(state = AppState)]
pub async fn login(
    State(auth): State<Arc<dyn AuthUseCase>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), StatusCode> {
    info!("Login attempt for user `{}`", &payload.username);

    let tokens = auth
        .login_user(&payload.username, &payload.password)
        .await
        .map_err(|e| {
            warn!("Login failed for user `{}`: {}", &payload.username, e);
            match e {
                AuthUseCaseError::InvalidUsername | AuthUseCaseError::InvalidPassword => {
                    StatusCode::UNAUTHORIZED
                }
                AuthUseCaseError::Internal(e) => {
                    error!("Login server error: {e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                _ => {
                    error!("Login unexpected error: {e}.");
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        })?;

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
        (status = 200, description = "Token refreshed", body = LoginResponse),
        (status = 401, description = "Unauthorized - Invalid credentials"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Authentication"]
)]
#[axum::debug_handler(state = AppState)]
pub async fn refresh_session(
    State(auth): State<Arc<dyn AuthUseCase>>,
    jar: CookieJar,
) -> Result<CookieJar, StatusCode> {

    // Get the refresh token from the cookie
    let refresh_token = jar.get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let new_tokens = auth
        .refresh_session(&refresh_token)
        .await
        .map_err(|e| {
            warn!("Session refresh failed: {e}");
            match e {
                AuthUseCaseError::InvalidRefreshToken => {
                    StatusCode::UNAUTHORIZED
                }
                AuthUseCaseError::Internal(e) => {
                    error!("Session refresh server error: {e}");
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                _ => {
                    error!("Session refresh unexpected error: {e}.");
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        })?;

    // Return the updated cookies
    let access_cookie = build_cookie("access_token", new_tokens.access_token, "/");
    let refresh_cookie = build_cookie("refresh_token", new_tokens.refresh_token, "/auth/refresh-session");

    Ok(jar.add(access_cookie).add(refresh_cookie))
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