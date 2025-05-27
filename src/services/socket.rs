use std::sync::Arc;

use axum::extract::ws::WebSocket;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
//Axum crate for websocket
use axum_ycrdt_websocket::broadcast::BroadcastGroup;
use axum_ycrdt_websocket::ws::{AxumSink, AxumStream};

use futures::StreamExt;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::services;
use crate::AppState;

#[derive(Deserialize)]
pub struct OpenFileParams {
    project_id: String,
    filename: String,
}

/**
 * socket routes
 * one is to get the folders
 * other is to get the code inside it
 */
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ws/:project_id", get(ws_handler))
        .route("/ws/:project_id/:filename", get(ws_handler_file))
        .route("/api/open-file/:project_id/:filename", post(open_file))
}

/**
 * socket to sync folder to users
 */
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(project_id): Path<String>,
    State(state): State<crate::AppState>,
) -> impl IntoResponse {
    if let Some(room) = state.rooms.get(&project_id) {
        let bcast = Arc::clone(&room);
        ws.on_upgrade(move |socket| peer(socket, bcast.clone()))
    } else {
        (
            StatusCode::NOT_FOUND,
            format!("Room with project_id '{}' not found", project_id),
        )
            .into_response()
    };
}

/**
 * socket to sync files to users
 */
async fn ws_handler_file(
    ws: WebSocketUpgrade,
    Path((project_id, filename)): Path<(String, String)>,
    State(state): State<crate::AppState>,
) -> impl IntoResponse {
    //get bcast from AppState
    let key = format!("{}/{}", project_id, filename);
    if let Some(room) = state.rooms.get(&key) {
        let bcast = Arc::clone(&room);
        ws.on_upgrade(move |socket| peer(socket, bcast.clone()))
    } else {
        (
            StatusCode::NOT_FOUND,
            format!("Room with filename '{}' not found", filename),
        )
            .into_response()
    };
}

// sharing message with peer
pub async fn peer(ws: WebSocket, bcast: Arc<BroadcastGroup>) {
    let (sink, stream) = ws.split();
    let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
    let stream = AxumStream::from(stream);

    let sub = bcast.subscribe(sink.clone(), stream);
    match sub.completed().await {
        Ok(_) => println!("broadcasting for channel finished successfully"),
        Err(e) => eprintln!("broadcasting for channel finished abruptly: {}", e),
    }
}

/**
 * function to open file
 */
#[axum::debug_handler]
pub async fn open_file(
    Path(OpenFileParams {
        project_id,
        filename,
    }): Path<OpenFileParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match services::crdt_ops::handle_open_file(&state, &project_id, &filename).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("Failed to open file");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
