use std::sync::Arc;

use axum::{
    Json,
    extract::State,
};
use axum_extra::extract::cookie::CookieJar;

use crate::{
    Config,
    AppState,
    prelude::*,
    application::{
        ports::input::AuthUseCase,
    },
    presentation::http::{
        ApiError,
        dtos::auth::{LoginRequest, LoginResponse},
    },
};
use super::build_cookie;

#[utoipa::path(
    post,
    path = "/auth/creds/login",
    description = "Logs in a user with username & password.",
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
    State(config): State<Arc<Config>>,
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
    let access_cookie = build_cookie("access_token", tokens.access_token, "/", &config.api.protocol);
    let refresh_cookie = build_cookie("refresh_token", tokens.refresh_token, "/auth/refresh-session", &config.api.protocol);

    info!("Login successful for user `{}`", &payload.username);

    let response = LoginResponse { message: "Login successful".to_string() };
    Ok((jar.add(access_cookie).add(refresh_cookie), Json(response)))
}