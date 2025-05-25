use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use dotenv::dotenv;
use tokio;

mod models;
mod services;

#[derive(Clone)]
struct AppState {
    db: mongodb::Database,
    redis: redis::aio::ConnectionManager,
    template_cache: DashMap<String, Arc<HashMap<String, String>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let db = services::db::connect_db().await?;
    let redis = services::redis::connect_redis().await?;
    let template_cache = DashMap::new();
    let state = AppState {
        db,
        redis,
        template_cache,
    };
    let app = Router::new()
        .route("/", get(|| async { "Go to /auth/github to log in" }))
        .route("/auth/github", get(services::auth::github_login))
        .route(
            "/auth/github/callback",
            get(services::auth::github_callback),
        )
        .route("/api/projects", post(services::project::create_project))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
