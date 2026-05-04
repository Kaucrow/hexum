use crate::application::ports::SessionRepository;
use super::RedisAdapter;
use async_trait::async_trait;
use redis::AsyncCommands;

#[async_trait]
impl SessionRepository for RedisAdapter {
    async fn store_session(&self, refresh_token: &str, user_id: &str, ttl_days: u64) -> Result<(), String> {
        let mut conn = self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| e.to_string())?;

        let ttl_seconds = ttl_days * 24 * 60 * 60;

        // Saves the key and sets the expiration
        let _: () = conn.set_ex(refresh_token, user_id, ttl_seconds)
            .await.map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn consume_session(&self, refresh_token: &str) -> Result<Option<String>, String> {
        let mut conn = self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| e.to_string())?;

        // Fetches the user_id and deletes the token
        let user_id: Option<String> = conn.get_del(refresh_token)
            .await
            .ok();

        Ok(user_id)
    }
}