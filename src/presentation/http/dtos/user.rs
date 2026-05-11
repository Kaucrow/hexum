use crate::prelude::*;
use crate::domain::user::User;
use utoipa::{ToSchema, IntoParams};

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    #[schema(example = "f47ac10b-58cc-4372-a567-0e02b2c3d479")]
    pub id: String,
    #[schema(example = "johndoe")]
    pub username: String,
    pub is_active: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.as_str().to_string(),
            is_active: user.is_active,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "johndoe")]
    pub username: String,
    #[schema(example = "password123")]
    pub password: String,
    #[schema(example = "johndoe@gmail.com")]
    pub email: String
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    #[schema(example = "Registration successful. A verification link has been sent to your email. Please click it to activate your account.")]
    pub message: String,
}

#[derive(Deserialize, IntoParams)]
pub struct VerifyQueryParams {
    /// The verification token sent via email
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyResponse {
    #[schema(example = "Account verification successful. You can now log in.")]
    pub message: String,
}