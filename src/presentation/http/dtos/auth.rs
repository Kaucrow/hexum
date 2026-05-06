use crate::prelude::*;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "alice_smith")]
    pub username: String,

    #[schema(example = "supersecret123")]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    #[schema(example = "Login successful")]
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct LogoutResponse {
    #[schema(example = "Logout successful")]
    pub message: String,
}