use std::{collections::HashMap, sync::Arc};

use crate::models::room_wrapper::RoomWrapper;
use anyhow::Result;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_ycrdt_websocket::broadcast::BroadcastGroup;
use dashmap::DashMap;
use dotenv::dotenv;
use tokio;
use tower_http::cors::{Any, CorsLayer};

use std::sync::atomic::{AtomicUsize, Ordering};
//Logging
use tracing_appender::rolling;
use tracing_subscriber::{fmt, EnvFilter};

mod models;
mod services;

#[derive(Clone)]
struct AppState {
    db: mongodb::Database,
    redis: redis::aio::ConnectionManager,
    template_cache: DashMap<String, Arc<HashMap<String, String>>>, //DashMap is used to provide a thread-Safe Hashmap
    rooms: Arc<DashMap<String, Arc<RoomWrapper>>>, //Arc used to provide bouncing between multiple threads
}

#[axum::debug_handler]
async fn list_rooms(State(state): State<AppState>) -> impl IntoResponse {
    let keys: Vec<String> = state
        .rooms
        .iter()
        .map(|entry| entry.key().clone())
        .collect();
    Json(keys)
}

#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = rolling::daily("logs", "myapp.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter("info")
        .init();

    tracing::info!("Logging initialised");

    dotenv().ok();

    let db = services::db::connect_db().await?;
    let redis = services::redis::connect_redis().await?;
    let template_cache = DashMap::new();
    let rooms = Arc::new(DashMap::new());

    let state = AppState {
        db,
        redis,
        template_cache,
        rooms,
    };

    //setting cors
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(list_rooms))
        .route("/auth/github", get(services::auth::github_login))
        .route(
            "/auth/github/callback",
            get(services::auth::github_callback),
        )
        .route("/api/projects", post(services::project::create_project))
        .merge(services::socket::router())
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listining on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
