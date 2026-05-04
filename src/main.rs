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
    application::services::AuthService,
    infrastructure::{PostgresAdapter, RedisSessionAdapter, PasetoSecurityAdapter},
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
    let redis_session_adapter = Arc::new(RedisSessionAdapter::new(&config)?);
    let paseto_security_adapter = Arc::new(PasetoSecurityAdapter::new(config.session.secret_key.clone()));

    let auth_service = AuthService::new(
        pg_adapter,
        redis_session_adapter,
        paseto_security_adapter,
    );

    let state = AppState {
        auth: Arc::new(auth_service),
    };

    let app = Router::new()
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