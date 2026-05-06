use std::str::FromStr;

use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    application::ports::output::{UserRepository, UserRepositoryError},
    domain::user::{User, Role, EmailAddress},
};
use super::{
    PostgresAdapter, QUERIES,
    dtos::user::UserDbRow,
};

impl PostgresAdapter {
    async fn do_add_new_user(&self, user: User) -> Result<(), LocalError> {
        let queries = QUERIES.get().expect("Queries not initialized.");

        let user_by_username = sqlx::query_as::<_, UserDbRow>(&queries.user.get_by_username)
            .bind(&user.username)
            .fetch_optional(&self.pool)
            .await?;

        if user_by_username.is_some() {
            return Err(LocalError::Logic(UserRepositoryError::UsernameInUse));
        }

        let user_by_email = sqlx::query_as::<_, UserDbRow>(&queries.user.get_by_email)
            .bind(user.email.as_str())
            .fetch_optional(&self.pool)
            .await?;

        if user_by_email.is_some() {
            return Err(LocalError::Logic(UserRepositoryError::EmailInUse));
        }

        let roles_strings: Vec<String> = user.roles
            .iter()
            .map(|r| r.to_string())
            .collect();

        sqlx::query(&queries.user.insert)
            .bind(user.id)
            .bind(user.username)
            .bind(user.passwd)
            .bind(user.email.as_str())
            .bind(roles_strings)
            .bind(user.is_active)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn do_activate_user_by_id(&self, id: &Uuid) -> Result<(), LocalError> {
        let queries = QUERIES.get().expect("Queries not initialized.");

        sqlx::query(&queries.user.activate_by_id)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn do_delete_user_by_id(&self, id: &Uuid) -> Result<(), LocalError> {
        let queries = QUERIES.get().expect("Queries not initialized.");

        sqlx::query(&queries.user.delete_by_id)
            .bind(id)
            .execute(&self.pool)
            .await?;

       Ok(())
    }
}

#[async_trait]
impl UserRepository for PostgresAdapter {
    async fn get_user_by_id(&self, id: &Uuid) -> Option<User> {
        let queries = QUERIES.get().expect("Queries not initialized.");

        let record = sqlx::query_as::<_, UserDbRow>(&queries.user.get_by_id)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .ok()??;

        let parsed_roles: Vec<Role> = record.roles
            .into_iter()
            .filter_map(|r| Role::from_str(&r).ok())
            .collect();

        let email_vo = EmailAddress::new(record.email).ok()?;

        Some(User {
            id: record.id,
            username: record.username,
            passwd: record.passwd,
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
            .filter_map(|r| Role::from_str(&r).ok())
            .collect();

        let email_vo = EmailAddress::new(record.email).ok()?;

        Some(User {
            id: record.id,
            username: record.username,
            passwd: record.passwd,
            email: email_vo,
            roles: parsed_roles,
            is_active: record.is_active,
        })
    }

    async fn add_new_user(&self, user: User) -> Result<(), UserRepositoryError> {
        Ok(self.do_add_new_user(user).await?)
    }

    async fn activate_user_by_id(&self, id: &Uuid) -> Result<(), UserRepositoryError> {
        Ok(self.do_activate_user_by_id(id).await?)
    }

    async fn delete_user_by_id(&self, id: &Uuid) -> Result<(), UserRepositoryError> {
        Ok(self.do_delete_user_by_id(id).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("{0}")]
    Logic(UserRepositoryError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl From<LocalError> for UserRepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Logic(e) => e,
            LocalError::Sqlx(e) => UserRepositoryError::Internal(e.to_string()),
        }
    }
}