use crate::{services::crdt_ops::RedisSnapshot, AppState};
use anyhow::Result;
use axum_ycrdt_websocket::broadcast::BroadcastGroup;
use redis::AsyncCommands;
use redis::{aio::ConnectionManager, Client};
use std::{env, sync::Arc};
use tracing::{error, info};
use yrs::Transact;
use yrs::{ReadTxn, StateVector};

/**
 * function to connect_redis
 */
pub async fn connect_redis() -> Result<ConnectionManager> {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL not set");
    let client = Client::open(redis_url)?;
    let manager = client.get_connection_manager().await?;
    Ok(manager)
}

/**
 * function to persist crdt snapshot to redis
 */

pub async fn persist_snapshot_to_redis(
    state: &AppState,
    project_id: &str,
    group: &Arc<BroadcastGroup>,
) -> Result<()> {
    info!("Persisting crdt snapshot to redis");
    let snapshot = {
        let awareness = group.awareness().read().await;
        let doc = awareness.doc();
        let txn = doc.transact();

        RedisSnapshot {
            update: txn.encode_state_as_update_v1(&StateVector::default()),
        }
    };
    let serialized = bincode::serialize(&snapshot)?;
    let mut redis_conn = state.redis.clone();
    let key = format!("project_snapshot:{}", project_id);
    redis_conn
        .set_ex::<_, _, ()>(key, serialized, 60 * 60 * 2)
        .await?;
    Ok(())
}

/**
 * function to get crdt snapshot from redis
 */
pub async fn get_snapshot_from_redis(state: &AppState, project_id: &str) -> Option<RedisSnapshot> {
    let key = format!("project_snapshot:{}", project_id);
    let mut redis_conn = state.redis.clone();
    if let Ok(bytes) = redis_conn.get::<_, Vec<u8>>(key).await {
        if let Ok(snapshot) = bincode::deserialize::<RedisSnapshot>(&bytes) {
            info!("Successfully loaded snapshot from Redis for {}", project_id);
            return Some(snapshot);
        } else {
            error!("Failed to deserialize Redis snapshot for {}", project_id);
        }
    } else {
        info!("No Redis snapshot found for {}", project_id);
    }
    None
}
