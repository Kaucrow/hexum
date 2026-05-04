use crate::domain::user::User;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn get_user_by_id(&self, id: &str) -> Option<User>;
    async fn get_user_by_token(&self, token: &str) -> Option<User>;
    async fn get_user_by_username(&self, username: &str) -> Option<User>;
}