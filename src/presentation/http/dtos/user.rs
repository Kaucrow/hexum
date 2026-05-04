use crate::prelude::*;
use crate::domain::user::User;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    #[schema(example = "usr_123abc")]
    pub id: String,
    #[schema(example = "alice_smith")]
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