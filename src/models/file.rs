use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct File {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub path: String,
    pub project_id: ObjectId,
    pub id_dir: bool,
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
