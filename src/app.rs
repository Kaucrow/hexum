use std::sync::Arc;

use axum::extract::FromRef;

use crate::{
    application::{
        ports::input::AuthUseCase,
    },
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub auth: Arc<dyn AuthUseCase>,
}