use axum_ycrdt_websocket::broadcast::BroadcastGroup;
use serde::{Deserialize, Serialize};
use std::sync::{atomic::AtomicUsize, Arc};

#[derive(Serialize, Deserialize)]
pub struct RedisSnapshot {
    pub update: Vec<u8>,
}

pub struct RoomWrapper {
    pub group: Arc<BroadcastGroup>,
    pub user_count: Arc<AtomicUsize>,
}
