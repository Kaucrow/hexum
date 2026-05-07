use utoipa::{
    Modify, OpenApi,
    openapi::security::SecurityScheme,
};
use super::{routes, dtos};

#[derive(OpenApi)]
#[openapi(
    info(description = "Scalable Axum backend using Hexagonal Architecture"),
    paths(
        // Management routes
        routes::management::manager_dashboard,
        routes::management::delete_database,

        // User routes
        routes::user::register,
        routes::user::verify,

        // Auth routes
        routes::auth::creds::login,
        routes::auth::oauth::oauth_login_ui,
        routes::auth::oauth::google_login,
        routes::auth::refresh_session,
        routes::auth::logout,
    ),
    components(
        schemas(
            // User DTOs
            dtos::user::UserResponse,
            dtos::user::RegisterRequest,
            dtos::user::RegisterResponse,

            // Auth DTOs
            dtos::auth::LoginRequest,
            dtos::auth::LoginResponse,
            dtos::auth::LogoutResponse,
        )
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDocs;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);

        components.add_security_scheme(
            "cookie_auth",
            SecurityScheme::ApiKey(
                utoipa::openapi::security::ApiKey::Cookie(
                    utoipa::openapi::security::ApiKeyValue::new("access_token"),
                ),
            ),
        );

        components.add_security_scheme(
            "refresh_cookie_auth",
            SecurityScheme::ApiKey(
                utoipa::openapi::security::ApiKey::Cookie(
                    utoipa::openapi::security::ApiKeyValue::new("refresh_token"),
                ),
            ),
        );
    }
}