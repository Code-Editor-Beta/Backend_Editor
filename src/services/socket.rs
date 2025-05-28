use std::sync::atomic::Ordering;
use std::sync::Arc;

use axum::extract::ws::WebSocket;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum_ycrdt_websocket::AwarenessRef;
use yrs::sync::Awareness;
use yrs::updates::decoder::Decode;
use yrs::{Doc, ReadTxn, StateVector};
//Axum crate for websocket
use axum_ycrdt_websocket::broadcast::BroadcastGroup;
use axum_ycrdt_websocket::ws::{AxumSink, AxumStream};

use crate::models::room_wrapper::{self, RoomWrapper};
use crate::services;
use crate::services::crdt_ops::RedisSnapshot;
use crate::services::redis;
use crate::AppState;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::atomic::AtomicUsize;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, trace, warn};
use yrs::Transact;

use super::redis::get_snapshot_from_redis;

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
    Router::new().route("/ws/:project_id", get(ws_folder_handler))
    // .route("/ws/:project_id/:filename", get(ws_handler_file))
    // .route("/api/open-file/:project_id/:filename", post(open_file))
}

/**
 * socket to sync folder to users
 * load initial snapshot to users
 */
async fn ws_folder_handler(
    ws: WebSocketUpgrade,
    Path(project_id): Path<String>,
    State(state): State<crate::AppState>,
) -> impl IntoResponse {
    // we just got the broadcast group (Dont keep a queue or backlog) and not the previous snapshot of crdt

    info!("user tries to connect for projecId: {}", project_id);

    //checking inMemory room if they persist

    let room = match state.rooms.get(&project_id) {
        Some(wrapper) => {
            info!("Fetched room from state");
            wrapper.clone()
        }
        None => {
            info!("Fetched room from redis");
            // try to load from Redis or Mongo
            match load_room_from_storage(&state, &project_id).await {
                Some(new_room) => {
                    state.rooms.insert(project_id.clone(), new_room.clone());
                    new_room
                }
                None => {
                    error!("No room found with project_id: {}", project_id);
                    return (
                        StatusCode::NOT_FOUND,
                        format!("Project not initiated with project_id {}", project_id),
                    )
                        .into_response();
                }
            }
        }
    };

    return ws.on_upgrade(move |socket| folder_peer(socket, room, state, project_id));
}

/**
 * Load the snapshot from redis and build the room
 */
async fn load_room_from_storage(state: &AppState, project_id: &str) -> Option<Arc<RoomWrapper>> {
    // Doing Redis for now
    if let Some(snapshot) = redis::get_snapshot_from_redis(state, project_id).await {
        let doc = Doc::new();

        {
            let mut txn = doc.transact_mut();
            let update = yrs::Update::decode_v1(&snapshot.update).ok()?;
            txn.apply_update(update);
        }
        let awareness: AwarenessRef = Arc::new(RwLock::new(Awareness::new(doc)));
        let group = Arc::new(BroadcastGroup::new(awareness.clone(), 32).await);

        let room = Arc::new(RoomWrapper {
            group: group.clone(),
            user_count: Arc::new(AtomicUsize::new(0)),
        });
        state.rooms.insert(project_id.to_string(), room.clone());
        return Some(room);
    }
    None
}

/**
 * send the snapshot of folder to users
 */
pub async fn folder_peer(
    ws: WebSocket,
    room: Arc<RoomWrapper>,
    state: AppState,
    project_id: String,
) {
    info!("Incrementing userCount in room");
    room.user_count.fetch_add(1, Ordering::SeqCst);
    let (sink, stream) = ws.split();
    let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
    let stream = AxumStream::from(stream);
    info!("Sharing snapshot of folders");
    let snapshot = {
        let awareness = room.group.awareness().read().await;
        let doc = awareness.doc();
        let txn = doc.transact();

        txn.encode_state_as_update_v1(&StateVector::default())
    };
    if sink.lock().await.send(snapshot).await.is_err() {
        return;
    }
    info!("Folder Snapshot Sent");
    let sub = room.group.subscribe(sink.clone(), stream);
    let _ = sub.completed().await;

    info!("user get disconneted");
    if room.user_count.fetch_sub(1, Ordering::SeqCst) == 1 {
        let _ = redis::persist_snapshot_to_redis(&state, &project_id, &room.group).await;
    }
}

// /**
//  * socket to sync files to users
//  */
// async fn ws_handler_file(
//     ws: WebSocketUpgrade,
//     Path((project_id, filename)): Path<(String, String)>,
//     State(state): State<crate::AppState>,
// ) -> impl IntoResponse {
//     //get bcast from AppState
//     let key = format!("{}/{}", project_id, filename);
//     if let Some(room) = state.rooms.get(&key) {
//         let bcast = Arc::clone(&room);
//         println!("Connecting to socket");
//         ws.on_upgrade(move |socket| file_peer(socket, bcast.clone()))
//     } else {
//         (
//             StatusCode::NOT_FOUND,
//             format!("Room with filename '{}' not found", filename),
//         )
//             .into_response()
//     };
// }

// pub async fn file_peer(ws: WebSocket, room: Arc<BroadcastGroup>) {
//     let (sink, stream) = ws.split();
//     let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
//     let stream = AxumStream::from(stream);
//     info!("Sharing snapshot of folders");
//     let snapshot = {
//         let awareness = room.awareness().read().await;
//         let doc = awareness.doc();
//         let txn = doc.transact();

//         txn.encode_state_as_update_v1(&StateVector::default())
//     };
//     if sink.lock().await.send(snapshot).await.is_err() {
//         return; // client disconnected before snapshot delivered
//     }
//     info!("Folder Snapshot Sent");
//     let sub = room.subscribe(sink.clone(), stream);
//     match sub.completed().await {
//         Ok(_) => println!("broadcasting for channel finished successfully"),
//         Err(e) => eprintln!("broadcasting for channel finished abruptly: {}", e),
//     }
// }

// /**
//  * function to open file
// //  */
// #[axum::debug_handler]
// pub async fn open_file(
//     Path(OpenFileParams {
//         project_id,
//         filename,
//     }): Path<OpenFileParams>,
//     State(state): State<AppState>,
// ) -> impl IntoResponse {
//     match services::crdt_ops::handle_open_file(&state, &project_id, &filename).await {
//         Ok(_) => StatusCode::OK,
//         Err(e) => {
//             eprintln!("Failed to open file");
//             StatusCode::INTERNAL_SERVER_ERROR
//         }
//     }
// }
