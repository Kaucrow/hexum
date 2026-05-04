use crate::{
    application::ports::output::UserRepository,
    domain::user::{User, Role, EmailAddress},
};
use super::{
    PostgresAdapter, QUERIES,
    dtos::user::UserDbRow,
};
use async_trait::async_trait;

#[async_trait]
impl UserRepository for PostgresAdapter {
    async fn get_user_by_id(&self, id: &str) -> Option<User> {
        let queries = QUERIES.get().expect("Queries not initialized.");

        let record = sqlx::query_as::<_, UserDbRow>(&queries.user.get_by_id)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .ok()??;

        let parsed_roles: Vec<Role> = record.roles
            .into_iter()
            .filter_map(|r| match r.as_str() {
                "Admin" => Some(Role::Admin),
                "Manager" => Some(Role::Manager),
                "BasicUser" => Some(Role::BasicUser),
                _ => None, 
            })
            .collect();

        let email_vo = EmailAddress::new(record.email).ok()?;

        Some(User {
            id: record.id,
            username: record.username,
            password: record.password,
            email: email_vo,
            roles: parsed_roles,
            is_active: record.is_active,
        })
    }

    async fn get_user_by_token(&self, token: &str) -> Option<User> {
        let queries = QUERIES.get().expect("Queries not initialized.");

        let record = sqlx::query_as::<_, UserDbRow>(&queries.user.get_by_token)
            .bind(token)
            .fetch_optional(&self.pool)
            .await
            .ok()??;

        let parsed_roles: Vec<Role> = record.roles
            .into_iter()
            .filter_map(|r| match r.as_str() {
                "Admin" => Some(Role::Admin),
                "Manager" => Some(Role::Manager),
                "BasicUser" => Some(Role::BasicUser),
                _ => None,
            })
            .collect();

        let email_vo = EmailAddress::new(record.email).ok()?;

        Some(User {
            id: record.id,
            username: record.username,
            password: record.password,
            email: email_vo,
            roles: parsed_roles,
            is_active: record.is_active,
        })
    }

    async fn get_user_by_username(&self, username: &str) -> Option<User> {
        let queries = QUERIES.get().expect("Queries not initialized.");

        let record = sqlx::query_as::<_, UserDbRow>(&queries.user.get_by_username)
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .ok()??;

        let parsed_roles: Vec<Role> = record.roles
            .into_iter()
            .filter_map(|r| match r.as_str() {
                "Admin" => Some(Role::Admin),
                "Manager" => Some(Role::Manager),
                "BasicUser" => Some(Role::BasicUser),
                _ => None, 
            })
            .collect();

        let email_vo = EmailAddress::new(record.email).ok()?;

        Some(User {
            id: record.id,
            username: record.username,
            password: record.password,
            email: email_vo,
            roles: parsed_roles,
            is_active: record.is_active,
        })
    }
}