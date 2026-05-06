use crate::prelude::*;
use crate::domain::user::User;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    #[schema(example = "usr_123abc")]
    pub id: String,
    #[schema(example = "john_doe")]
    pub username: String,
    pub is_active: bool,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            is_active: user.is_active,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "john_doe")]
    pub username: String,
    #[schema(example = "password123")]
    pub password: String,
    #[schema(example = "johndoe@email.com")]
    pub email: String
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    #[schema(example = "Registration successful. A verification link has been sent to your email. Please click it to activate your account.")]
    pub message: String,
}