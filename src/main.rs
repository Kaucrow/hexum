use std::sync::Arc;

use anyhow::Result;
use axum::{Router, routing::{get, post, delete}};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use hexum::{
    AppState,
    Environment,
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
        GoogleAuthAdapter,
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
    let paseto_security_adapter = Arc::new(PasetoSecurityAdapter::new()?);
    let google_auth_adapter = Arc::new(
    GoogleAuthAdapter::new(
            config.oauth.google.client_id.clone(),
            config.oauth.google.client_secret.clone(),
            config.oauth.redirect_url(config.frontend.url())),
    );

    let auth_service = AuthService::new(
        pg_adapter.clone(),
        redis_session_adapter.clone(),
        paseto_security_adapter.clone(),
        google_auth_adapter,
    );

    let redis_verification_adapter = Arc::new(RedisVerificationAdapter::new(&config).await?);
    let lettre_email_adapter = Arc::new(LettreEmailAdapter::new(&config)?);

    let user_service = UserService::new(
        pg_adapter,
        redis_verification_adapter,
        paseto_security_adapter,
        lettre_email_adapter,
    );

    let state = AppState {
        config: config.clone(),
        auth: Arc::new(auth_service),
        user: Arc::new(user_service),
    };

    let mut app = Router::new()
        .route("/user/register", post(routes::user::register))
        .route("/user/verify", get(routes::user::verify))
        .route("/auth/creds/login", post(routes::auth::creds::login))
        .route("/auth/oauth/google/login", post(routes::auth::oauth::google_login))
        .route("/auth/refresh-session", post(routes::auth::refresh_session));

    if matches!(config.environment, Environment::Development) {
        app = app
            .route("/user/verify-ui", get(routes::user::verify_ui))
            .route("/auth/oauth/login-ui", get(routes::auth::oauth::oauth_login_ui))
            .route("/auth/oauth/callback-ui", get(routes::auth::oauth::oauth_callback_ui))
            .route("/dashboard", get(routes::management::manager_dashboard))
            .route("/nuke", delete(routes::management::delete_database))
    }

    let app = app
        .merge(Scalar::with_url(format!("/{}", config.api.docs_endpoint), http::ApiDocs::openapi()))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", config.api.host, config.api.port)
    ).await.unwrap();

    info!("App running on {} mode.", config.environment);
    info!("API listening on {}...", config.api.url());
    info!("View API docs at {}{}...", config.api.url(), config.api.docs_endpoint);

    axum::serve(listener, app).await.unwrap();

    Ok(())
}