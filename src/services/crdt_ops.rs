use crate::models::{project::Project, room_wrapper::RoomWrapper};
use crate::services::redis::persist_snapshot_to_redis;
use crate::AppState;
use anyhow::Result;
use axum::{http::StatusCode, response::IntoResponse};
use axum_ycrdt_websocket::broadcast::BroadcastGroup;
use axum_ycrdt_websocket::AwarenessRef;
use bincode;
use mongodb::bson::doc;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};
use yrs::sync::Awareness;
use yrs::{Doc, Map, Text, Transact};
use yrs::{ReadTxn, StateVector};

#[derive(Serialize, Deserialize)]
pub struct RedisSnapshot {
    pub update: Vec<u8>,
}

/**
 * only insert the folder name in the Yjs doc
 * will load the content when needed
 *
 */

pub async fn initialize_crdt_room(
    state: &AppState,
    project_id: &str,
    files: &HashMap<String, String>,
) -> Result<()> {
    info!(
        "Initializing crdt Doc for project with projectID {}",
        project_id
    );
    let doc = Doc::new();
    {
        let map = doc.get_or_insert_map("files");
        let mut txn = doc.transact_mut();

        for filename in files.keys() {
            map.insert(&mut txn, filename.to_string(), "");
        }
    }

    let awareness: AwarenessRef = Arc::new(RwLock::new(Awareness::new(doc)));
    let group = Arc::new(BroadcastGroup::new(awareness.clone(), 32).await);

    let room = Arc::new(RoomWrapper {
        group: group.clone(),
        user_count: Arc::new(AtomicUsize::new(0)),
    });

    info!("Inserting room with projectId: {}", project_id);

    state.rooms.insert(project_id.to_string(), room);
    info!(
        "Inserted room for project: {}, total rooms: {}",
        project_id,
        state.rooms.len()
    );
    persist_snapshot_to_redis(&state, project_id, &group).await?;

    Ok(())
}

// /**
//  * function to open files and create a room for that file edit
//  */
// pub async fn handle_open_file(
//     state: &AppState,
//     project_id: &str,
//     filename: &str,
// ) -> Result<impl IntoResponse, (StatusCode, String)> {
//     println!("inside handle_open_file");
//     let collection = state.db.collection::<mongodb::bson::Document>("projects");
//     let result = collection
//         .find_one(doc! { "project_name": &project_id })
//         .await
//         .map_err(|err| {
//             (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("Database error: {}", err),
//             )
//         })?;
//     if result.is_none() {
//         return Err((
//             StatusCode::NOT_FOUND,
//             format!("Project '{}' not found", project_id),
//         ));
//     }
//     let project: Project = match mongodb::bson::from_document(result.unwrap()) {
//         Ok(project) => project,
//         Err(error) => {
//             return Err((
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("Error reading project data: {}", error),
//             ));
//         }
//     };
//     let content = match project.files.get(filename) {
//         Some(content) => content,
//         None => {
//             return Err((
//                 StatusCode::NOT_FOUND,
//                 format!("File '{}' not found in project '{}'", filename, project_id),
//             ));
//         }
//     };
//     let file_room_key = format!("{}/{}", project_id, filename);
//     if !state.rooms.contains_key(&file_room_key) {
//         let doc = Doc::new();
//         {
//             let text = doc.get_or_insert_text("content");
//             let mut txn = doc.transact_mut();
//             text.push(&mut txn, content);
//         }

//         // Set up the CRDT room for real-time editing
//         let awareness = Arc::new(RwLock::new(Awareness::new(doc)));
//         let group = Arc::new(BroadcastGroup::new(awareness, 32).await);
//         state.rooms.insert(file_room_key.clone(), group);
//         println!("Created file CRDT room: {}", file_room_key);
//     }

//     Ok(())
// }
