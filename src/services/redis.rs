use anyhow::Result;
use redis::{aio::ConnectionManager, Client};
use std::env;

/**
 * function to connect_redis
 */
pub async fn connect_redis() -> Result<ConnectionManager> {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL not set");
    let client = Client::open(redis_url)?;
    let manager = client.get_connection_manager().await?;
    Ok(manager)
}
