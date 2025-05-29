use crate::services::crdt_ops::models::RoomWrapper;
use crate::services::redis::crdt_snapshot_to_redis;
use crate::AppState;
use anyhow::Result;
use axum_ycrdt_websocket::broadcast::BroadcastGroup;
use axum_ycrdt_websocket::AwarenessRef;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use yrs::sync::Awareness;
use yrs::{Doc, Map, Transact};

/**
 * only insert the folder name in the Yjs doc
 * will load the content when needed
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
    crdt_snapshot_to_redis::persist_snapshot_to_redis(&state, project_id, &group).await?;

    Ok(())
}
