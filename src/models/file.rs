use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone)]
pub struct File {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub path: String,
    pub project_id: ObjectId,
    pub content: String,
    pub language: String,
    pub versions: Vec<FileVersion>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileVersion {
    pub content: String,
    pub timestamp: DateTime<Utc>,
}