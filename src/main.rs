use std::sync::Arc;

use anyhow::Result;
use axum::{Router, routing::{get, post, delete}};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use hexum::{
    AppState,
    get_config,
    prelude::*,
    telemetry,
    application::services::{AuthService, UserService},
    infrastructure::{
        PostgresAdapter,
        RedisSessionAdapter,
        PasetoSecurityAdapter,
        RedisVerificationAdapter,
        LettreEmailAdapter,
    },
    presentation::http::{self, routes},
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = Arc::new(get_config()?);

    // Init the tracing subscriber
    let (subscriber, _guard) = telemetry::get_subscriber(&config).await?;
    telemetry::init(subscriber);

    let pg_adapter = Arc::new(PostgresAdapter::new(&config).await?);
    let redis_session_adapter = Arc::new(RedisSessionAdapter::new(&config).await?);
    let paseto_security_adapter = Arc::new(PasetoSecurityAdapter::new(config.session.secret_key.clone()));

    let auth_service = AuthService::new(
        pg_adapter.clone(),
        redis_session_adapter.clone(),
        paseto_security_adapter.clone(),
    );

    let redis_verification_adapter = Arc::new(RedisVerificationAdapter::new(&config).await?);
    let lettre_email_adapter = Arc::new(LettreEmailAdapter::new()?);

    let user_service = UserService::new(
        pg_adapter,
        redis_verification_adapter,
        paseto_security_adapter,
        lettre_email_adapter,
    );

    let state = AppState {
        auth: Arc::new(auth_service),
        user: Arc::new(user_service),
    };

    let app = Router::new()
        .route("/user/register", post(routes::user::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/refresh-session", post(routes::auth::refresh_session))
        .route("/dashboard", get(routes::management::manager_dashboard))
        .route("/nuke", delete(routes::management::delete_database))
        .merge(Scalar::with_url(format!("/{}", config.api.docs_endpoint), http::ApiDocs::openapi()))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", config.api.host, config.api.port)
    ).await.unwrap();
    info!("API listening on {}...", config.api.url());
    info!("View docs at {}/{}...", config.api.url(), config.api.docs_endpoint);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}