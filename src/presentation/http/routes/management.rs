use crate::{
    AppState,
    domain::user::Role,
    presentation::http::{
        extractors::AuthenticatedUser,
        dtos::user::UserResponse,
    },
};
use axum::{http::StatusCode, response::IntoResponse, Json};

// Endpoint open to Admins and Managers
#[utoipa::path(
    get,
    path = "/manager/dashboard",
    responses(
        (status = 200, description = "Dashboard loaded successfully", body = UserResponse),
        (status = 401, description = "Unauthorized - Invalid or missing access token")
    ),
    tags = ["Management"]
)]
#[axum::debug_handler(state = AppState)]
pub async fn manager_dashboard(
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<impl IntoResponse, StatusCode> {

    // Check roles
    if !user.has_any_role(&[Role::Admin, Role::Manager]) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(serde_json::json!({ "msg": format!("Welcome back, {}", user.username) })))
}

#[utoipa::path(
    get,
    path = "/admin/delete-db",
    responses(
        (status = 200, description = "Database deleted successfully"),
        (status = 401, description = "Unauthorized - Invalid or missing access token")
    ),
    tags = ["Management"]
)]
#[axum::debug_handler(state = AppState)]
// Admins-only endpoint
pub async fn delete_database(
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<impl IntoResponse, StatusCode> {

    if !user.has_any_role(&[Role::Admin]) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok("Database deleted. owo")
}