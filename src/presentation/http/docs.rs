use utoipa::{
    Modify, OpenApi,
    openapi::security::SecurityScheme,
};
use super::{routes, dtos};

#[derive(OpenApi)]
#[openapi(
    info(description = "Scalable Axum backend using Hexagonal Architecture"),
    paths(
        routes::management::manager_dashboard,
        routes::management::delete_database,
        routes::auth::login,
    ),
    components(
        schemas(
            dtos::user::UserResponse,
            dtos::auth::LoginRequest,
            dtos::auth::LoginResponse
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
    }
}