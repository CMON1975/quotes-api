mod auth;
mod config;
mod db;
mod errors;
mod models;
mod routes;

use routes::AppState;
use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let config = config::Config::load();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let state = AppState {
        pool,
        api_key: config.api_key.clone(),
    };

    let app = routes::build_router(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind address");

    tracing::info!("listening on {}", addr);

    axum::serve(listener, app).await.expect("server error");
}
