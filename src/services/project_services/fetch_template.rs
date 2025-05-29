use crate::services::{brotili, project_services::read_template_from_disk};
use crate::AppState;
use anyhow::Result;
use rmp_serde::encode;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tracing::info;

/**
 * Returns pathBuf for the provided framework
 * add more frameworks in future to work on here
 */
fn template_root(framework: &str) -> Result<PathBuf> {
    match framework {
        "react" => Ok(PathBuf::from("templates/react-template")),
        _ => anyhow::bail!("Unsupported framework: {framework}"),
    }
}

/**
 * fetch the template from 3 parts
 * first check the inMemory cache
 * then check redis
 * then check Disk
 */
pub async fn fetch_template(
    framework: &str,
    state: &AppState,
) -> anyhow::Result<Arc<HashMap<String, String>>> {
    // check in_memory Cache
    info!("check for template in memory cache");
    if let Some(t) = state.template_cache.get(framework) {
        return Ok(t);
    }

    // check in redis
    info!("check for template in redis when not found in cache");
    let redis_key = format!("template:{framework}");
    let mut redis = state.redis.clone();
    if let Ok(Some(blob)) = redis::cmd("GET")
        .arg(&redis_key)
        .query_async::<Option<Vec<u8>>>(&mut redis)
        .await
    {
        let decompressed = brotili::decompress::decompress_brotli(&blob);
        if let Ok(map) = rmp_serde::from_slice::<HashMap<String, String>>(&decompressed) {
            let arc = Arc::new(map);
            state
                .template_cache
                .insert(framework.to_owned(), arc.clone());
            return Ok(arc);
        }
    }

    // check in Disk
    info!("check for template in disk when not found in redis+cache");
    let root = template_root(framework)?;
    let map = read_template_from_disk::read_template_from_disk(root).await?;
    let arc = Arc::new(map);
    state
        .template_cache
        .insert(framework.to_string(), arc.clone());
    let arc_for_redis = arc.clone();

    //creating brotli compressed version of tempalte
    info!("Creating brotli compressed file");
    let serialized = encode::to_vec(&*arc_for_redis).expect("msgpack serialize");
    let compressed = brotili::compress::compress_brotli(&serialized);
    // store in Redis for 1 hr
    info!("Storing template to redis for 1 hr for persistance");
    tokio::spawn(async move {
        let _: () = redis::cmd("SETEX")
            .arg(redis_key)
            .arg(3600u32)
            .arg(compressed)
            .query_async(&mut redis)
            .await
            .unwrap_or(());
    });

    Ok(arc)
}
