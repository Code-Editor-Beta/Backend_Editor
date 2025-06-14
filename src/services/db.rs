use super::super::models::user::User;
use crate::AppState;
use anyhow::Result;
use axum::extract::State;
use chrono::Utc;
use mongodb::{bson::oid::ObjectId, Client, Collection, Database};
use std::env;

/**
 * function to connect_db
 */
pub async fn connect_db() -> Result<Database> {
    let uri = env::var("MONGODB_URI").expect("You must set the ENV for MongoDB_URI");

    let client = Client::with_uri_str(&uri).await?;
    let db = client.database("backendRust");
    Ok(db)
}

/**
 * function to create_user entry in db
 */
pub async fn create_user(user: User, State(state): State<AppState>) -> Result<String, String> {
    let db = state.db;
    let collection: Collection<User> = db.collection("Users");

    let new_user = User {
        id: ObjectId::new(),
        github_id: user.github_id,
        name: user.name,
        email: user.email,
        avatar_url: user.avatar_url,
        access_token: user.access_token,
        projects: vec![],
        created_at: Utc::now(),
    };
    collection
        .insert_one(&new_user)
        .await
        .map_err(|err| format!("Failed to insert user: {}", err))?;

    Ok(new_user.id.to_hex())
}
