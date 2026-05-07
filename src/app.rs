use std::sync::Arc;

use axum::extract::FromRef;

use crate::{
    Config,
    application::{
        ports::input::{AuthUseCase, UserUseCase}
    },
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Arc<Config>,
    pub auth: Arc<dyn AuthUseCase>,
    pub user: Arc<dyn UserUseCase>,
}