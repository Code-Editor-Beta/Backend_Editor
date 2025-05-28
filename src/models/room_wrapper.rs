use axum_ycrdt_websocket::broadcast::BroadcastGroup;
use std::sync::{atomic::AtomicUsize, Arc};
pub struct RoomWrapper {
    pub group: Arc<BroadcastGroup>,
    pub user_count: Arc<AtomicUsize>,
}
