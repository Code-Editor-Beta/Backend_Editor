use anyhow::Result;
use axum::{routing::get, Router};
use dotenv::dotenv;
use tokio;

mod models;
mod services;

#[derive(Clone)]
struct AppState {
    db: mongodb::Database,
    redis: redis::aio::ConnectionManager,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let db = services::db::connect_db().await?;
    let redis = services::redis::connect_redis().await?;
    let state = AppState { db, redis };
    let app = Router::new()
        .route("/", get(|| async { "Go to /auth/github to log in" }))
        .route("/auth/github", get(services::auth::github_login))
        .route(
            "/auth/github/callback",
            get(services::auth::github_callback),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
