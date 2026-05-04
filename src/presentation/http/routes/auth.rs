use crate::{
    Config,
    AppState,
    application::{
        ports::{SessionRepository, UserRepository},
        auth::{generate_access_token, generate_refresh_token, verify_password}
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
    State(config): State<Arc<Config>>,
    State(session_repo): State<Arc<dyn SessionRepository>>,
    State(user_repo): State<Arc<dyn UserRepository>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), StatusCode> {

    let user = user_repo.get_user_by_username(&payload.username).await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_password(&payload.password, &user.password) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let access_token = generate_access_token(&user.id, &config.session.secret_key).unwrap();
    let refresh_token = generate_refresh_token();

    session_repo
    .store_session(&refresh_token, &user.id, 7)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Attach cookies
    // Access token goes to the root path ("/")
    let access_cookie = build_cookie("access_token", access_token, "/");
    // Refresh token strictly goes to the refresh endpoint to save bandwidth and increase security
    let refresh_cookie = build_cookie("refresh_token", refresh_token, "/auth/refresh");

    let response = LoginResponse { message: "Success".to_string() };
    Ok((jar.add(access_cookie).add(refresh_cookie), Json(response)))
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    responses(
        (status = 200, description = "Token refreshed", body = LoginResponse),
        (status = 401, description = "Unauthorized - Invalid credentials"),
        (status = 500, description = "Internal Server Error")
    ),
    tags = ["Authentication"]
)]
#[axum::debug_handler(state = AppState)]
pub async fn refresh(
    State(config): State<Arc<Config>>,
    State(session_repo): State<Arc<dyn SessionRepository>>,
    State(user_repo): State<Arc<dyn UserRepository>>,
    jar: CookieJar,
) -> Result<CookieJar, StatusCode> {

    // Get the refresh token from the cookie
    let refresh_token = jar.get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Consume it from Redis (verifies & deletes the token)
    let user_id = session_repo.consume_session(&refresh_token).await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Fetch the user (ensure they aren't deactivated/deleted since last login)
    let user = user_repo.get_user_by_id(&user_id).await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Generate brand new tokens
    let new_access = generate_access_token(&user.id, &config.session.secret_key).unwrap();
    let new_refresh = generate_refresh_token();

    // Store the new refresh token in Redis
    session_repo
    .store_session(&new_refresh, &user.id, 7)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Return the updated cookies
    let access_cookie = build_cookie("access_token", new_access, "/");
    let refresh_cookie = build_cookie("refresh_token", new_refresh, "/auth/refresh");

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