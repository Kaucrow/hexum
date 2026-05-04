use crate::{
    Config,
    application::ports::{UserRepository, SessionRepository},
    infrastructure::{PostgresAdapter, RedisAdapter},
};
use std::sync::Arc;
use axum::extract::FromRef;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub app_config: Arc<Config>,
    pub pg_adapter: Arc<PostgresAdapter>,
    pub redis_adapter: Arc<RedisAdapter>,
}

macro_rules! impl_repo_extractors {
    ($field:ident => $($repo_trait:path),+ $(,)?) => {
        $(
            impl axum::extract::FromRef<AppState> for std::sync::Arc<dyn $repo_trait> {
                fn from_ref(state: &AppState) -> Self {
                    state.$field.clone() as std::sync::Arc<dyn $repo_trait>
                }
            }
        )+
    };
}

impl_repo_extractors!(
    pg_adapter => UserRepository,
);

impl_repo_extractors!(
    redis_adapter => SessionRepository,
);