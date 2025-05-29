use std::{collections::HashMap, sync::Arc};

use crate::services::{auth, crdt_ops, db, project_services, redis as rd, socket};
use anyhow::Result;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dashmap::DashMap;
use dotenv::dotenv;
use tokio;
use tower_http::cors::{Any, CorsLayer};
//used for caching
use moka::sync::Cache;
//Logging
use tracing_appender::rolling;

mod models;
mod services;
use tracing::info;
#[derive(Clone)]
struct AppState {
    db: mongodb::Database,
    redis: redis::aio::ConnectionManager,
    template_cache: Cache<String, Arc<HashMap<String, String>>>,
    rooms: Arc<DashMap<String, Arc<crdt_ops::models::RoomWrapper>>>,
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

    let db = db::connect_db::connect_db().await?;
    let redis = rd::connect_redis::connect_redis().await?;
    info!("Intializing cache memory");
    let template_cache: Cache<String, Arc<HashMap<String, String>>> = Cache::builder()
        .time_to_live(std::time::Duration::from_secs(3600))
        .max_capacity(32)
        .build();
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
        .route("/auth/github", get(auth::github_login::github_login))
        .route(
            "/auth/github/callback",
            get(auth::github_callback::github_callback),
        )
        .route(
            "/api/projects",
            post(project_services::create_project::create_project),
        )
        .route(
            "/ws/:project_id",
            get(socket::folder_socket::ws_folder_handler),
        )
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listining on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
