use crate::{models::project::Project, AppState};
use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use chrono::Utc;
use mongodb::bson::{doc, oid::ObjectId};
use rmp_serde::encode;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::task;
use walkdir::WalkDir;

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub user_id: String,
    pub project_name: String,
    pub framework: String,
}

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
 * this reads all files under the root and return paths+content
 * if file is text -> store text
 * if file is binary -> store binary
 */
async fn read_template_from_disk(root: PathBuf) -> anyhow::Result<HashMap<String, String>> {
    // use move here to give ownership of root and all variables to the thread we are shifting the process
    task::spawn_blocking(move || {
        let mut map = HashMap::new();

        for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let abs = entry.path();
                let rel = abs
                    .strip_prefix(&root)?
                    .to_string_lossy()
                    .replace('\\', "/");

                let bytes = std::fs::read(abs)?;
                let value = match String::from_utf8(bytes.clone()) {
                    Ok(text) => text,
                    Err(_) => format!("__BIN__{}", B64.encode(&bytes)),
                };

                map.insert(rel, value);
            }
        }
        Ok(map)
    })
    .await?
}

/**
 * fetch the template from 3 parts
 * first check the inMemory location
 * then check redis
 * then check Disk
 */
async fn fetch_template(
    framework: &str,
    state: &AppState,
) -> anyhow::Result<Arc<HashMap<String, String>>> {
    // check in_memory location
    if let Some(t) = state.template_cache.get(framework) {
        return Ok(t.clone());
    }

    // check in redis
    let redis_key = format!("template:{framework}");
    let mut redis = state.redis.clone();
    if let Ok(Some(blob)) = redis::cmd("GET")
        .arg(&redis_key)
        .query_async::<Option<Vec<u8>>>(&mut redis)
        .await
    {
        if let Ok(map) = rmp_serde::from_slice::<HashMap<String, String>>(&blob) {
            let arc = Arc::new(map);
            state
                .template_cache
                .insert(framework.to_owned(), arc.clone());
            return Ok(arc);
        }
    }

    // check in Disk
    let root = template_root(framework)?;
    let map = read_template_from_disk(root).await?;
    let arc = Arc::new(map);
    state
        .template_cache
        .insert(framework.to_owned(), arc.clone());
    let arc_for_redis = arc.clone();

    // store in Redis for 1 hr
    tokio::spawn(async move {
        let _: () = redis::cmd("SETEX")
            .arg(redis_key)
            .arg(3600u32)
            .arg(encode::to_vec(&*arc_for_redis).expect("msgpack serialize"))
            .query_async(&mut redis)
            .await
            .unwrap_or(());
    });

    Ok(arc)
}

/**
 * api endpoint to create_project
 */
pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> impl IntoResponse {
    let template_res = fetch_template(&payload.framework, &state).await;

    let template = match template_res {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("template error: {e}")).into_response();
        }
    };

    let project = Project {
        id: ObjectId::new(),
        user_id: payload.user_id.clone(),
        project_name: payload.project_name.clone(),
        framework: payload.framework.to_lowercase(),
        created_at: Utc::now(),
        files: (*template).clone(),
    };

    let collection = state.db.collection::<mongodb::bson::Document>("projects");
    let serialized = mongodb::bson::to_document(&project).unwrap();

    match collection.insert_one(serialized).await {
        Ok(_) => (StatusCode::OK, Json(project)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB insert error: {}", e),
        )
            .into_response(),
    }
}
