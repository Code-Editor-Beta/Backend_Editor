use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone)]
pub struct User{
    #[serde(rename="_id")]
    pub id: ObjectId,
    pub github_id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    #[serde(skip_serializing)]
    pub access_token: Option<String>,
    pub projects:Vec<ObjectId>,
    pub created_at: DateTime<Utc>
}

