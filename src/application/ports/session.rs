use async_trait::async_trait;

#[async_trait]
pub trait SessionRepository: Send + Sync + 'static {
    // Stores the refresh token and associates it with a user ID for a given number of days
    async fn store_session(&self, refresh_token: &str, user_id: &str, ttl_days: u64) -> Result<(), String>;

    // Fetches the user ID associated with the token and deletes the token
    async fn consume_session(&self, refresh_token: &str) -> Result<Option<String>, String>;
}