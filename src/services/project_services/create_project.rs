use crate::services::{crdt_ops::initialize_room, project_services::fetch_template};
use crate::{models::project::Project, AppState};

use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    response::Response,
};

use chrono::Utc;
use mongodb::bson::{doc, oid::ObjectId};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub user_id: String,
    pub project_name: String,
    pub framework: String,
}

/**
 * api endpoint to create_project
 */
#[axum::debug_handler]
pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Response, (StatusCode, String)> {
    let template_res = fetch_template::fetch_template(&payload.framework, &state).await;

    let template = template_res.map_err(|_err| {
        (
            StatusCode::BAD_REQUEST,
            format!("Error fetching Default Template"),
        )
    })?;

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

    collection.insert_one(serialized).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB insert error: {}", e),
        )
    })?;

    //Initialise the Socket room for users
    initialize_room::initialize_crdt_room(
        &state,
        &project.project_name.to_string(),
        &project.files,
    )
    .await
    .map_err(|_| (StatusCode::BAD_REQUEST, "Error creating room".to_string()))?;

    Ok(Json("User created successfully").into_response())
}
