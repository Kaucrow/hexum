use hexum::{
    AppState,
    get_config,
    prelude::*,
    telemetry,
    infrastructure::{PostgresAdapter, RedisAdapter},
    presentation::http::{self, routes},
};

use anyhow::Result;
use axum::{Router, routing::{get, post, delete}};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = Arc::new(get_config()?);

    // Init the tracing subscriber
    let (subscriber, _guard) = telemetry::get_subscriber(&config).await?;
    telemetry::init(subscriber);

    let pg_adapter = PostgresAdapter::new(&config).await?;
    let redis_adapter = RedisAdapter::new(&config)?;

    let state = AppState {
        app_config: config.clone(),
        pg_adapter: Arc::new(pg_adapter),
        redis_adapter: Arc::new(redis_adapter),
    };

    let app = Router::new()
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/refresh", post(routes::auth::refresh))
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